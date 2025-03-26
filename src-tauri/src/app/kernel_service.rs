use crate::process::manager::ProcessManager;
use std::sync::Arc;
use tracing::{error, info};
use serde_json::json;
use std::path::Path;
use crate::utils::app_util::get_work_dir;
use crate::utils::file_util::{unzip_file};
use std::os::windows::process::CommandExt;
use tauri::Emitter;
use crate::app::constants::{paths, process, messages};
use tauri::{Runtime, Window};
use tokio::task;
use tokio::sync::mpsc;
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use serde_json::Value;

// Глобальный менеджер процессов
lazy_static::lazy_static! {
    pub(crate) static ref PROCESS_MANAGER: Arc<ProcessManager> = Arc::new(ProcessManager::new());
}

// Проверка версии ядра
#[tauri::command]
pub async fn check_kernel_version() -> Result<String, String> {
    let kernel_path = paths::get_kernel_path();

    if !kernel_path.exists() {
        return Err(messages::ERR_KERNEL_NOT_FOUND.to_string());
    }

    let output = std::process::Command::new(kernel_path)
        .arg("version")
        .creation_flags(process::CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("{}: {}", messages::ERR_VERSION_CHECK_FAILED, e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{}: {}", messages::ERR_GET_VERSION_FAILED, error));
    }

    let version_info = String::from_utf8_lossy(&output.stdout);
    Ok(version_info.to_string())
}

// Запуск ядра
#[tauri::command]
pub async fn start_kernel() -> Result<(), String> {
    PROCESS_MANAGER.start().await.map_err(|e| e.to_string())
}

// Остановка ядра
#[tauri::command]
pub async fn stop_kernel() -> Result<(), String> {
    PROCESS_MANAGER.stop().await.map_err(|e| e.to_string())
}

// Перезапуск ядра
#[tauri::command]
pub async fn restart_kernel() -> Result<(), String> {
    PROCESS_MANAGER.restart().await.map_err(|e| e.to_string())
}

// Получение статуса процесса
#[tauri::command]
pub async fn get_process_status() -> serde_json::Value {
    let info = PROCESS_MANAGER.get_status().await;
    json!({
        "status": info.status,
        "pid": info.pid,
        "last_error": info.last_error
    })
}

// Получение использования памяти
#[tauri::command]
pub async fn get_memory_usage() -> Result<String, String> {
    let output = std::process::Command::new("wmic")
        .args([
            "process",
            "where",
            "name='sing-box.exe'",
            "get",
            "WorkingSetSize",
        ])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| e.to_string())?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    if lines.len() < 2 {
        return Ok("0".to_string());
    }

    let memory = lines[1].trim();
    if memory.is_empty() {
        Ok("0".to_string())
    } else {
        Ok((memory.parse::<u64>().unwrap_or(0) / 1024 / 1024).to_string())
    }
}

// Загрузка последней версии ядра
#[tauri::command]
pub async fn download_latest_kernel(window: tauri::Window) -> Result<(), String> {
    let work_dir = get_work_dir();
    info!("Текущий рабочий каталог: {}", work_dir);

    let path = Path::new(&work_dir).join("sing-box/");
    info!("Целевой каталог загрузки: {}", path.display());

    // Если каталог существует, сначала проверяем, является ли он допустимым каталогом
    if path.exists() {
        if !path.is_dir() {
            error!("Путь sing-box существует, но не является каталогом");
            return Err("Путь sing-box существует, но не является каталогом".toString());
        }
    }

    // Убедитесь, что каталог существует
    if let Err(e) = std::fs::create_dir_all(&path) {
        error!("Не удалось создать каталог: {}", e);
        return Err(format!("Не удалось создать каталог: {}", e));
    }
    info!("Каталог загрузки существует");

    info!("Подготовка к загрузке последней версии...");
    // Отправка события прогресса
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "checking",
            "progress": 0,
            "message": "Получение информации о последней версии..."
        }),
    );

    // Получение информации о последней версии
    let client = reqwest::Client::new();
    let releases_url = "https://api.github.com/repos/SagerNet/sing-box/releases/latest";
    let response = client
        .get(releases_url)
        .header("User-Agent", "sing-box-windows")
        .send()
        .await
        .map_err(|e| format!("Не удалось получить информацию о версии: {}", e))?;

    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Не удалось разобрать информацию о версии: {}", e))?;

    // Получение номера версии
    let version = release["tag_name"]
        .as_str()
        .ok_or("Не удалось получить номер версии")?
        .trim_start_matches('v')
        .toString();

    // Получение текущей платформы и архитектуры системы
    let platform = std::env::consts::OS;
    let mut arch = std::env::consts::ARCH;
    if arch == "x86_64" {
        arch = "amd64";
    }

    // Формирование имени целевого файла
    let target_asset_name = format!("sing-box-{}-{}-{}.zip", version, platform, arch);
    info!("Имя целевого файла: {}", target_asset_name);

    // Поиск ресурса для Windows
    let assets = release["assets"].as_array().ok_or("Не удалось получить ресурсы выпуска")?;
    let asset = assets
        .iter()
        .find(|asset| {
            if let Some(name) = asset["name"].as_str() {
                name.contains("windows-amd64") && name.ends_with(".zip")
            } else {
                false
            }
        })
        .ok_or("Не удалось найти ресурс для Windows")?;

    // Получение ссылки для загрузки
    let original_url = asset["browser_download_url"]
        .as_str()
        .ok_or("Не удалось получить ссылку для загрузки")?;

    info!("Найдена ссылка для загрузки: {}", original_url);

    let download_path = Path::new(&path).join(&target_asset_name);
    info!("Целевой путь загрузки: {}", download_path.display());

    // Отправка события прогресса
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "downloading",
            "progress": 20,
            "message": format!("Начало загрузки файла: {}", target_asset_name)
        }),
    );

    // Загрузка файла
    let window_clone = window.clone();
    if let Err(e) = crate::utils::file_util::download_with_fallback(
        original_url, 
        download_path.to_str().unwrap(), 
        move |progress| {
            let real_progress = 20 + (progress as f64 * 0.6) as u32; // 20-80% прогресса для загрузки
            let _ = window_clone.emit(
                "download-progress",
                json!({
                    "status": "downloading",
                    "progress": real_progress,
                    "message": format!("Загрузка: {}%", progress)
                }),
            );
        }).await {
        error!("Ошибка загрузки: {}", e);
        return Err(format!(
            "Ошибка загрузки: {}.\nВы можете попробовать загрузить вручную:\n1. Перейдите на https://github.com/SagerNet/sing-box/releases/latest\n2. Загрузите {}\n3. Распакуйте и поместите файлы в каталог {}/sing-box/",
            e, target_asset_name, get_work_dir()
        ));
    }

    // Распаковка файла
    info!("Начало распаковки файла...");
    // Отправка события прогресса
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "extracting",
            "progress": 80,
            "message": "Распаковка файла..."
        }),
    );

    let out_path = Path::new(&work_dir).join("sing-box");
    match unzip_file(download_path.to_str().unwrap(), out_path.to_str().unwrap()).await {
        Ok(_) => {
            info!("Ядро загружено и распаковано в: {}", out_path.display());
            // Отправка события завершения
            let _ = window.emit(
                "download-progress",
                json!({
                    "status": "completed",
                    "progress": 100,
                    "message": "Загрузка завершена!"
                }),
            );
        }
        Err(e) => {
            error!("Ошибка распаковки файла: {}", e);
            return Err(format!("Ошибка распаковки файла: {}", e));
        }
    }

    Ok(())
}

/// Запуск WebSocket релея данных
#[tauri::command]
pub async fn start_websocket_relay<R: Runtime>(window: Window<R>) -> Result<(), String> {
    // Запуск четырех различных типов WebSocket релеев
    start_traffic_relay(window.clone()).await?;
    start_memory_relay(window.clone()).await?;
    start_logs_relay(window.clone()).await?;
    start_connections_relay(window.clone()).await?;
    
    Ok(())
}

/// Запуск релея данных трафика
async fn start_traffic_relay<R: Runtime>(window: Window<R>) -> Result<(), String> {
    let window_clone = window.clone();
    let (tx, mut rx) = mpsc::channel(32);
    let token = crate::app::proxy_service::get_api_token();
    
    // Запуск WebSocket соединения и задачи обработки данных
    let _handle = task::spawn(async move {
        let url = Url::parse(&format!("ws://127.0.0.1:12081/traffic?token={}", token)).unwrap();
        
        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                let (mut _write, mut read) = ws_stream.split();
                
                // Постоянное чтение сообщений WebSocket
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                let _ = tx.send(data).await;
                            }
                        },
                        Ok(Message::Close(_)) => break,
                        Err(e) => {
                            error!("Ошибка чтения данных WebSocket трафика: {}", e);
                            break;
                        },
                        _ => {}
                    }
                }
            },
            Err(e) => {
                error!("Ошибка подключения WebSocket трафика: {}", e);
            }
        }
    });
    
    // Запуск задачи отправки событий
    task::spawn(async move {
        while let Some(data) = rx.recv().await {
            let _ = window_clone.emit("traffic-data", data);
        }
    });
    
    Ok(())
}

/// Запуск релея данных памяти
async fn start_memory_relay<R: Runtime>(window: Window<R>) -> Result<(), String> {
    let window_clone = window.clone();
    let (tx, mut rx) = mpsc::channel(32);
    let token = crate::app::proxy_service::get_api_token();
    
    // Запуск WebSocket соединения и задачи обработки данных
    let _handle = task::spawn(async move {
        let url = Url::parse(&format!("ws://127.0.0.1:12081/memory?token={}", token)).unwrap();
        
        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                let (mut _write, mut read) = ws_stream.split();
                
                // Постоянное чтение сообщений WebSocket
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                let _ = tx.send(data).await;
                            }
                        },
                        Ok(Message::Close(_)) => break,
                        Err(e) => {
                            error!("Ошибка чтения данных WebSocket памяти: {}", e);
                            break;
                        },
                        _ => {}
                    }
                }
            },
            Err(e) => {
                error!("Ошибка подключения WebSocket памяти: {}", e);
            }
        }
    });
    
    // Запуск задачи отправки событий
    task::spawn(async move {
        while let Some(data) = rx.recv().await {
            let _ = window_clone.emit("memory-data", data);
        }
    });
    
    Ok(())
}

/// Запуск релея данных логов
async fn start_logs_relay<R: Runtime>(window: Window<R>) -> Result<(), String> {
    let window_clone = window.clone();
    let (tx, mut rx) = mpsc::channel(32);
    let token = crate::app::proxy_service::get_api_token();
    
    // Запуск WebSocket соединения и задачи обработки данных
    let _handle = task::spawn(async move {
        let url = Url::parse(&format!("ws://127.0.0.1:12081/logs?token={}", token)).unwrap();
        
        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                let (mut _write, mut read) = ws_stream.split();
                
                // Постоянное чтение сообщений WebSocket
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                let _ = tx.send(data).await;
                            }
                        },
                        Ok(Message::Close(_)) => break,
                        Err(e) => {
                            error!("Ошибка чтения данных WebSocket лога: {}", e);
                            break;
                        },
                        _ => {}
                    }
                }
            },
            Err(e) => {
                error!("Ошибка подключения WebSocket лога: {}", e);
            }
        }
    });
    
    // Запуск задачи отправки событий
    task::spawn(async move {
        while let Some(data) = rx.recv().await {
            let _ = window_clone.emit("log-data", data);
        }
    });
    
    Ok(())
}

/// Запуск релея данных соединений
async fn start_connections_relay<R: Runtime>(window: Window<R>) -> Result<(), String> {
    let window_clone = window.clone();
    let (tx, mut rx) = mpsc::channel(32);
    let token = crate::app::proxy_service::get_api_token();
    
    // Запуск WebSocket соединения и задачи обработки данных
    let _handle = task::spawn(async move {
        let url = Url::parse(&format!("ws://127.0.0.1:12081/connections?token={}", token)).unwrap();
        
        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                let (mut _write, mut read) = ws_stream.split();
                
                // Постоянное чтение сообщений WebSocket
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                let _ = tx.send(data).await;
                            }
                        },
                        Ok(Message::Close(_)) => break,
                        Err(e) => {
                            error!("Ошибка чтения данных WebSocket соединений: {}", e);
                            break;
                        },
                        _ => {}
                    }
                }
            },
            Err(e) => {
                error!("Ошибка подключения WebSocket соединений: {}", e);
            }
        }
    });
    
    // Запуск задачи отправки событий
    task::spawn(async move {
        while let Some(data) = rx.recv().await {
            let _ = window_clone.emit("connections-data", data);
        }
    });
    
    Ok(())
}