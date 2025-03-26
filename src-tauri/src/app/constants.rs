//! Определение констант приложения
//! 
//! Этот файл содержит все определения констант, используемых в приложении
//! Централизованное управление константами упрощает их изменение и обслуживание

/// Константы, связанные с процессами
pub mod process {
    /// Флаг для скрытия окна консоли при создании процесса в Windows
    pub const CREATE_NO_WINDOW: u32 = 0x08000000;
    
    /// Константы таймаута и задержки процесса (в секундах)
    pub const GRACEFUL_TIMEOUT: u64 = 5;
    pub const HEALTH_CHECK_INTERVAL: u64 = 30;
    pub const MAX_RESTART_ATTEMPTS: u32 = 3;
    pub const RESTART_DELAY: u64 = 1;
}

/// Константы путей файлов
pub mod paths {
    use std::path::PathBuf;
    use crate::utils::app_util::get_work_dir;

    /// Получение пути к исполняемому файлу Sing-Box
    pub fn get_kernel_path() -> PathBuf {
        let work_dir = get_work_dir();
        PathBuf::from(&work_dir).join("sing-box").join("sing-box.exe")
    }

    /// Получение рабочего каталога Sing-Box
    pub fn get_kernel_work_dir() -> PathBuf {
        let work_dir = get_work_dir();
        PathBuf::from(&work_dir).join("sing-box")
    }

    /// Получение пути к файлу конфигурации
    pub fn get_config_path() -> PathBuf {
        let work_dir = get_work_dir();
        PathBuf::from(&work_dir).join("sing-box").join("config.json")
    }
}

/// Константы сети
pub mod network {
    /// Адрес по умолчанию для прослушивания
    pub const DEFAULT_LISTEN_ADDRESS: &str = "0.0.0.0";
    
    /// Порт прокси по умолчанию
    pub const DEFAULT_PROXY_PORT: u16 = 12080;
    
    /// Адрес Clash API по умолчанию
    pub const DEFAULT_CLASH_API_ADDRESS: &str = "127.0.0.1";
    
    /// Порт Clash API по умолчанию
    pub const DEFAULT_CLASH_API_PORT: u16 = 12081;
    
    /// Токен API по умолчанию
    pub const DEFAULT_API_TOKEN: &str = "";
    
    /// Таймаут сетевого запроса (в секундах)
    pub const HTTP_TIMEOUT_SECONDS: u64 = 30;
}

/// Константы API
pub mod api {
    /// URL GitHub API
    pub const GITHUB_API_URL: &str = "https://api.github.com/repos/xinggaoya/sing-box-windows/releases/latest";
    
    /// Пользовательский агент
    pub const USER_AGENT: &str = "sing-box-windows";
}

/// Константы сообщений
pub mod messages {
    // Сообщения об ошибках
    pub const ERR_KERNEL_NOT_FOUND: &str = "Файл ядра не найден";
    pub const ERR_VERSION_CHECK_FAILED: &str = "Не удалось выполнить проверку версии";
    pub const ERR_GET_VERSION_FAILED: &str = "Не удалось получить информацию о версии";
    pub const ERR_CONFIG_READ_FAILED: &str = "Не удалось прочитать файл конфигурации";
    pub const ERR_DOWNLOAD_FAILED: &str = "Не удалось загрузить";
    pub const ERR_SUBSCRIPTION_FAILED: &str = "Не удалось загрузить подписку";
    pub const ERR_PROCESS_SUBSCRIPTION_FAILED: &str = "Не удалось обработать содержимое подписки";
    pub const ERR_GET_EXE_PATH_FAILED: &str = "Не удалось получить путь к текущей программе";
    pub const ERR_RESTART_FAILED: &str = "Не удалось перезапустить";
    pub const ERR_INVALID_CONFIG: &str = "Недействительный файл конфигурации";
    pub const ERR_PROCESS_ALREADY_RUNNING: &str = "Процесс уже запущен";
    pub const ERR_PROCESS_NOT_RUNNING: &str = "Процесс не запущен";
    pub const ERR_PROCESS_START_FAILED: &str = "Не удалось запустить процесс";
    pub const ERR_PROCESS_STOP_FAILED: &str = "Не удалось остановить процесс";
    pub const ERR_HTTP_CLIENT_FAILED: &str = "Не удалось создать HTTP-клиент";
    pub const ERR_REQUEST_FAILED: &str = "Запрос не удался";
    pub const ERR_SERVER_ERROR: &str = "Сервер вернул ошибочный статус";
    pub const ERR_FILE_SIZE_UNKNOWN: &str = "Не удалось получить размер файла";
    pub const ERR_CREATE_DIR_FAILED: &str = "Не удалось создать каталог";
    pub const ERR_CREATE_FILE_FAILED: &str = "Не удалось создать файл";
    pub const ERR_OPEN_FILE_FAILED: &str = "Не удалось открыть файл";
    pub const ERR_READ_ARCHIVE_FAILED: &str = "Не удалось прочитать архив";
    pub const ERR_EXTRACT_FILE_FAILED: &str = "Не удалось извлечь файл";
    pub const ERR_INVALID_FILENAME: &str = "Недействительное имя файла";
    pub const ERR_WRITE_FILE_FAILED: &str = "Не удалось записать файл";
    pub const ERR_READ_FILE_FAILED: &str = "Не удалось прочитать файл";
    pub const ERR_KEY_NOT_FOUND: &str = "Ключ не найден";
    
    // Информационные сообщения
    pub const INFO_PROCESS_STARTED: &str = "Процесс успешно запущен";
    pub const INFO_PROCESS_STOPPED: &str = "Процесс остановлен";
    pub const INFO_SYSTEM_PROXY_DISABLED: &str = "Системный прокси отключен";
    pub const INFO_CONFIG_CHECK_PASSED: &str = "Проверка файла конфигурации пройдена";
    pub const INFO_PROXY_MODE_ENABLED: &str = "Режим прокси включен";
    pub const INFO_DOWNLOAD_STARTED: &str = "Начата загрузка файла";
    pub const INFO_UNZIP_STARTED: &str = "Начата распаковка файла";
    pub const INFO_EXTRACTING_FILE: &str = "Распаковка файла";
}

/// Конфигурационные константы
pub mod config {
    /// Тег по умолчанию для входящих соединений
    pub const DEFAULT_INBOUND_TAG: &str = "mixed-in";
    
    /// Тип по умолчанию для входящих соединений
    pub const DEFAULT_INBOUND_TYPE: &str = "mixed";
}

/// Константы для логирования
pub mod log {
    /// Уровень логирования по умолчанию
    pub const DEFAULT_LEVEL: &str = "debug";
    
    /// Директория для логов
    pub const DEFAULT_DIR: &str = "logs";
    
    /// Префикс имени файла лога
    pub const DEFAULT_FILE_PREFIX: &str = "app";
    
    /// Типы ротации логов
    pub mod rotation {
        pub const HOURLY: &str = "hourly";
        pub const DAILY: &str = "daily";
        pub const NEVER: &str = "never";
        pub const DEFAULT: &str = "daily";
    }
    
    /// Максимальный размер файла лога (в МБ)
    pub const DEFAULT_MAX_FILE_SIZE: u64 = 100;
    
    /// Максимальное количество файлов лога
    pub const DEFAULT_MAX_FILES: u32 = 30;
}

/// Константы реестра
pub mod registry {
    /// Путь к реестру настроек интернета в Windows
    pub const INTERNET_SETTINGS: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";
    
    /// Имя ключа включения прокси
    pub const PROXY_ENABLE: &str = "ProxyEnable";
    
    /// Имя ключа сервера прокси
    pub const PROXY_SERVER: &str = "ProxyServer";
}

/// Конфигурация сервера по умолчанию
pub mod server {
    /// Адрес хоста по умолчанию
    pub const DEFAULT_HOST: &str = "127.0.0.1";
    
    /// Порт по умолчанию
    pub const DEFAULT_PORT: u16 = 8080;
}

/// Конфигурация базы данных по умолчанию
pub mod database {
    /// URL подключения к базе данных по умолчанию
    pub const DEFAULT_URL: &str = "sqlite://data.db";
}

/// Конфигурация JWT аутентификации
pub mod jwt {
    /// Секретный ключ по умолчанию (внимание: в производственной среде используйте безопасный случайный ключ)
    pub const DEFAULT_SECRET: &str = "your-secret-key";
    
    /// Время истечения по умолчанию (в секундах)
    pub const DEFAULT_EXPIRATION: i64 = 86400; // 24 часа
}

/// Конфигурация ограничения скорости
pub mod rate_limit {
    /// Время окна по умолчанию (в секундах)
    pub const DEFAULT_WINDOW_SECS: u64 = 60;
    
    /// Максимальное количество запросов по умолчанию
    pub const DEFAULT_MAX_REQUESTS: u64 = 100;
}