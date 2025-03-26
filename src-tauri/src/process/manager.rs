use super::{ProcessError, ProcessInfo, ProcessStatus, Result};
use crate::utils::app_util::get_work_dir;
use std::os::windows::process::CommandExt;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use crate::utils::proxy_util::disable_system_proxy;
use crate::app::constants::{paths, process, messages};

pub struct ProcessManager {
    process_info: Arc<RwLock<ProcessInfo>>,
    child_process: Arc<RwLock<Option<tokio::process::Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            process_info: Arc::new(RwLock::new(ProcessInfo {
                pid: None,
                status: ProcessStatus::Stopped,
                last_error: None,
            })),
            child_process: Arc::new(RwLock::new(None)),
        }
    }

    // Получение статуса процесса
    pub async fn get_status(&self) -> ProcessInfo {
        self.process_info.read().await.clone()
    }

    // Проверка наличия других процессов sing-box
    #[allow(dead_code)]
    async fn check_other_sing_box_process(&self) -> Option<u32> {
        // Получение собственного PID для исключения
        let self_pid = {
            let info = self.process_info.read().await;
            info.pid
        };
        
        match std::process::Command::new("tasklist")
            .arg("/FI")
            .arg("IMAGENAME eq sing-box.exe")
            .arg("/FO")
            .arg("CSV")
            .arg("/NH")
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.trim().is_empty() {
                    return None; // Нет процессов sing-box
                }
                
                // Разбор всех PID процессов sing-box
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        // Извлечение PID и преобразование в u32
                        if let Ok(pid) = parts[1].trim_matches('"').parse::<u32>() {
                            // Исключение собственного PID
                            if self_pid != Some(pid) {
                                info!("Обнаружен другой процесс sing-box: PID={}", pid);
                                return Some(pid);
                            }
                        }
                    }
                }
                None
            }
            Err(e) => {
                error!("Не удалось проверить наличие других процессов sing-box: {}", e);
                None
            }
        }
    }

    // Сброс состояния процесса
    async fn reset_process_state(&self) {
        let mut info = self.process_info.write().await;
        info.status = ProcessStatus::Stopped;
        info.pid = None;
        info.last_error = None;

        let mut process = self.child_process.write().await;
        *process = None;
    }

    // Проверка, запущен ли процесс
    pub async fn is_running(&self) -> bool {
        self._is_running(false).await
    }

    // Проверка перед запуском
    async fn pre_start_check(&self) -> Result<()> {
        // Принудительная проверка состояния процесса, чтобы убедиться, что состояние соответствует действительности
        let force_check = true;
        let is_running = self._is_running(force_check).await;
        
        // Если текущий экземпляр процесса запущен, сначала попытаться его остановить
        if is_running {
            info!("Обнаружено, что ядро приложения уже запущено, попытка принудительной остановки");
            match self.force_stop().await {
                Ok(_) => info!("Успешно остановлено текущее запущенное ядро"),
                Err(e) => {
                    warn!("Не удалось остановить текущее запущенное ядро: {}", e);
                    // Даже если не удалось остановить, продолжаем попытку следующего шага
                }
            }
            
            // Сброс состояния, чтобы обеспечить чистую среду для запуска
            self.reset_process_state().await;
        }

        // Проверка наличия других процессов sing-box (по имени процесса)
        if self.is_process_running_by_name("sing-box.exe").await {
            info!("Обнаружены другие процессы sing-box, попытка принудительной остановки всех экземпляров");
            if let Err(e) = self.kill_process_by_name("sing-box.exe").await {
                warn!("Не удалось остановить некоторые процессы sing-box: {}", e);
            }
            
            // Ожидание полной остановки процессов
            sleep(Duration::from_secs(1)).await;
            
            // Повторная проверка, остановлены ли все процессы
            if self.is_process_running_by_name("sing-box.exe").await {
                warn!("Некоторые процессы sing-box все еще запущены, продолжаем попытку запуска");
            }
        }

        // Проверка конфигурационного файла
        self.check_config().await?;

        Ok(())
    }

    // Проверка, запущен ли процесс по имени
    async fn is_process_running_by_name(&self, process_name: &str) -> bool {
        let query = format!("IMAGENAME eq {}", process_name);
        
        match std::process::Command::new("tasklist")
            .arg("/FI")
            .arg(query)
            .arg("/FO")
            .arg("CSV")
            .arg("/NH")
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                !output_str.is_empty() && output_str.contains(process_name)
            }
            Err(e) => {
                error!("Не удалось проверить имя процесса: {}", e);
                false
            }
        }
    }

    // Принудительное завершение всех процессов по имени
    async fn kill_process_by_name(&self, process_name: &str) -> std::io::Result<()> {
        // Использование команды taskkill /IM для принудительного завершения всех процессов по имени
        let output = std::process::Command::new("taskkill")
            .arg("/F") // Принудительное завершение
            .arg("/IM")
            .arg(process_name)
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("Нет запущенных задач") {
                // Игнорирование ошибки "Нет запущенных задач", это означает, что не найдено соответствующих процессов
                return Ok(());
            }
            error!("Не удалось завершить процесс: {}", error);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other, 
                format!("Не удалось завершить процесс: {}", error)
            ));
        }
        
        info!("Все процессы {} завершены", process_name);
        Ok(())
    }

    // Проверка конфигурационного файла
    async fn check_config(&self) -> Result<()> {
        info!("Текущий рабочий каталог: {}", get_work_dir());
        
        // Проверка существования конфигурационного файла
        let config_path = paths::get_config_path();
        if !config_path.exists() {
            return Err(ProcessError::ConfigError(messages::ERR_CONFIG_READ_FAILED.to_string()));
        }

        // Проверка конфигурационного файла
        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| ProcessError::ConfigError(format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e)))?;

        // Разбор JSON
        let json_result: serde_json::Result<serde_json::Value> = serde_json::from_str(&config_str);
        if let Err(e) = json_result {
            return Err(ProcessError::ConfigError(format!("Ошибка формата JSON в конфигурационном файле: {}", e)));
        }

        // Проверка конфигурации - использование встроенной функции проверки sing-box
        let kernel_path = paths::get_kernel_path();
        let output = std::process::Command::new(&kernel_path)
            .arg("check")
            .arg("-c")
            .arg(&config_path)
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()
            .map_err(|e| ProcessError::ConfigError(format!("Не удалось проверить конфигурацию: {}", e)))?;

        if !output.status.success() {
            let error_output = String::from_utf8_lossy(&output.stderr);
            return Err(ProcessError::ConfigError(format!("Недействительная конфигурация: {}", error_output)));
        }

        info!("Проверка конфигурационного файла пройдена");
        
        // Если вывод пуст, конфигурация действительна
        Ok(())
    }

    // Запуск процесса
    pub async fn start(&self) -> Result<()> {
        // Обновление статуса на "Запуск"
        {
            let mut info = self.process_info.write().await;
            info.status = ProcessStatus::Starting;
            info.last_error = None;
        }

        // Проверка перед запуском
        if let Err(e) = self.pre_start_check().await {
            self.handle_error(e.clone()).await?;
            return Err(e);
        }

        // Получение рабочего каталога и пути к ядру
        let kernel_work_dir = paths::get_kernel_work_dir();
        let kernel_path = paths::get_kernel_path();

        // Запуск процесса
        let child = match Command::new(kernel_path.to_str().unwrap())
            .arg("run")
            .arg("-D")
            .arg(kernel_work_dir.to_str().unwrap())
            .creation_flags(process::CREATE_NO_WINDOW)
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                let err = ProcessError::StartFailed(format!("Не удалось запустить: {}", e));
                self.handle_error(err.clone()).await?;
                return Err(err);
            }
        };

        // Обновление информации о процессе
        {
            let mut info = self.process_info.write().await;
            info.pid = Some(child.id().unwrap_or(0));
            info.status = ProcessStatus::Starting;
        }

        // Сохранение дочернего процесса
        {
            let mut process = self.child_process.write().await;
            *process = Some(child);
        }

        info!("{}", messages::INFO_PROCESS_STARTED);
        Ok(())
    }

    // Остановка процесса
    pub async fn stop(&self) -> Result<()> {
        // Проверка статуса процесса
        let status = self.get_status().await.status;
        if matches!(status, ProcessStatus::Stopped) {
            return Ok(());
        }

        // Обновление статуса на "Остановка"
        {
            let mut info = self.process_info.write().await;
            info.status = ProcessStatus::Stopping;
        }

        // Сначала попытка корректной остановки процесса
        // let stop_result = self.graceful_stop().await;
        
        // Если корректная остановка не удалась, принудительное завершение
        // if stop_result.is_err() {
            warn!("Превышено время ожидания остановки процесса, попытка принудительного завершения");
            self.force_stop().await?;
        // }
        
        // Отключение системного прокси
        if let Err(e) = disable_system_proxy() {
            warn!("Не удалось отключить системный прокси: {}", e);
        } else {
            info!("{}", messages::INFO_SYSTEM_PROXY_DISABLED);
        }

        // Обновление статуса процесса
        {
            let mut info = self.process_info.write().await;
            info.status = ProcessStatus::Stopped;
            info.pid = None;
        }

        info!("{}", messages::INFO_PROCESS_STOPPED);
        Ok(())
    }

    // Отправка сигнала остановки
    #[allow(dead_code)]
    fn send_signal(&self, pid: u32) -> std::io::Result<()> {
        std::process::Command::new("taskkill")
            .arg("/PID")
            .arg(pid.to_string())
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()?;
        Ok(())
    }

    // Принудительное завершение процесса
    fn kill_process(&self, pid: u32) -> std::io::Result<()> {
        std::process::Command::new("taskkill")
            .arg("/F")
            .arg("/PID")
            .arg(pid.to_string())
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()?;
        Ok(())
    }

    // Перезапуск процесса с принудительной остановкой
    pub async fn restart(&self) -> Result<()> {
        self.stop().await?;
        // Ожидание 1 секунду
        sleep(Duration::from_secs(1)).await;
        self.start().await?;
        Ok(())
    }

    // Обработка ошибок
    async fn handle_error(&self, err: ProcessError) -> Result<()> {
        let mut info = self.process_info.write().await;
        info.status = ProcessStatus::Failed(err.to_string());
        info.last_error = Some(err.to_string());
        error!("Ошибка процесса: {}", err);
        Ok(())
    }

    // Корректная остановка процесса
    #[allow(dead_code)]
    async fn graceful_stop(&self) -> Result<()> {
        let pid = {
            let info = self.process_info.read().await;
            info.pid.ok_or(ProcessError::NotRunning)?
        };

        // Попытка отправки сигнала нормальной остановки
        if let Err(e) = self.send_signal(pid) {
            return Err(ProcessError::StopFailed(format!("Не удалось отправить сигнал остановки: {}", e)));
        }

        // Время ожидания остановки процесса
        let timeout = Duration::from_secs(process::GRACEFUL_TIMEOUT);
        let start = std::time::Instant::now();

        // Ожидание остановки процесса
        while self.check_process_exists(Some(pid)).await {
            if start.elapsed() > timeout {
                return Err(ProcessError::StopFailed("Превышено время ожидания остановки процесса".to_string()));
            }
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    // Принудительная остановка процесса
    async fn force_stop(&self) -> Result<()> {
        let pid = {
            let info = self.process_info.read().await;
            info.pid.ok_or(ProcessError::NotRunning)?
        };

        // Принудительное завершение процесса
        if let Err(e) = self.kill_process(pid) {
            return Err(ProcessError::StopFailed(format!("Не удалось принудительно завершить: {}", e)));
        }

        // Кратковременное ожидание для уверенности, что процесс завершен
        sleep(Duration::from_millis(500)).await;
        
        // Проверка, завершен ли процесс
        if self.check_process_exists(Some(pid)).await {
            return Err(ProcessError::StopFailed("Не удалось принудительно завершить, процесс все еще запущен".to_string()));
        }

        Ok(())
    }

    // Проверка, существует ли процесс
    async fn check_process_exists(&self, pid: Option<u32>) -> bool {
        if let Some(pid) = pid {
            // Использование конкретного PID для точного соответствия
            let query = format!("PID eq {}", pid);
            
            match std::process::Command::new("tasklist")
                .arg("/FI")
                .arg(query)
                .arg("/FO")
                .arg("CSV")
                .arg("/NH") // Не показывать заголовок
                .creation_flags(process::CREATE_NO_WINDOW)
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    // Проверка, содержит ли вывод PID
                    // Формат вывода должен быть "sing-box.exe","PID",...
                    if output_str.is_empty() {
                        return false;
                    }
                    
                    // Более строгая проверка соответствия PID
                    output_str.contains(&format!(",\"{}\"", pid))
                }
                Err(_) => {
                    error!("Не удалось проверить состояние процесса");
                    false
                }
            }
        } else {
            false
        }
    }

    // Внутренняя функция проверки, запущен ли процесс, с возможностью принудительной проверки
    async fn _is_running(&self, force_check: bool) -> bool {
        // Сначала проверка информации о процессе
        let info = self.process_info.read().await;
        let status_running = matches!(
            info.status,
            ProcessStatus::Running | ProcessStatus::Starting
        );

        // Если статус показывает, что процесс не запущен, и не требуется принудительная проверка, возвращаем false
        if !status_running && !force_check {
            return false;
        }

        // Если нет PID, процесс не запущен
        if info.pid.is_none() {
            if status_running {
                // Несоответствие статуса, требуется сброс
                drop(info); // Освобождение блокировки чтения
                self.reset_process_state().await;
                warn!("Статус процесса показывает, что он запущен, но нет PID, статус сброшен");
            }
            return false;
        }

        // Проверка, существует ли процесс
        let pid = info.pid.unwrap(); // Безопасно, так как уже проверено на is_none
        let exists = self.check_process_exists(Some(pid)).await;
        
        if !exists && status_running {
            // Процесс не существует, но статус показывает, что он запущен, сброс статуса
            drop(info); // Освобождение блокировки чтения
            self.reset_process_state().await;
            warn!("Статус процесса показывает, что он запущен (PID: {}), но процесс не существует, статус сброшен", pid);
            return false;
        }

        // Возвращаем фактическое состояние процесса
        exists
    }
}

