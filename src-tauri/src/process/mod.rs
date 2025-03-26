use serde::{Deserialize, Serialize};
use std::fmt;
use crate::app::constants::process as process_constants;

pub mod manager;

// Статус процесса
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed(String),
}

// Ошибки процесса
#[derive(Debug, Clone)]
pub enum ProcessError {
    AlreadyRunning,
    NotRunning,
    StartFailed(String),
    StopFailed(String),
    StatusCheckFailed(String),
    ConfigError(String),
    SystemError(String),
    PermissionError(String),
    NetworkError(String),
    Unknown(String),
}

impl From<std::io::Error> for ProcessError {
    fn from(err: std::io::Error) -> Self {
        ProcessError::SystemError(err.to_string())
    }
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessError::AlreadyRunning => write!(f, "Процесс уже запущен"),
            ProcessError::NotRunning => write!(f, "Процесс не запущен"),
            ProcessError::StartFailed(msg) => write!(f, "Ошибка запуска: {}", msg),
            ProcessError::StopFailed(msg) => write!(f, "Ошибка остановки: {}", msg),
            ProcessError::StatusCheckFailed(msg) => write!(f, "Ошибка проверки состояния процесса: {}", msg),
            ProcessError::ConfigError(msg) => write!(f, "Ошибка конфигурации: {}", msg),
            ProcessError::SystemError(msg) => write!(f, "Системная ошибка: {}", msg),
            ProcessError::PermissionError(msg) => write!(f, "Ошибка прав доступа: {}", msg),
            ProcessError::NetworkError(msg) => write!(f, "Сетевая ошибка: {}", msg),
            ProcessError::Unknown(msg) => write!(f, "Неизвестная ошибка: {}", msg),
        }
    }
}

impl std::error::Error for ProcessError {}

// Информация о процессе
#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: Option<u32>,
    pub status: ProcessStatus,
    pub last_error: Option<String>,
}

pub type Result<T> = std::result::Result<T, ProcessError>;

// Конфигурация процесса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub graceful_timeout: u64,      // Время ожидания корректного завершения (секунды)
    pub health_check_interval: u64, // Интервал проверки состояния (секунды)
    pub max_restart_attempts: u32,  // Максимальное количество попыток перезапуска
    pub restart_delay: u64,         // Задержка перед перезапуском (секунды)
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            graceful_timeout: process_constants::GRACEFUL_TIMEOUT,
            health_check_interval: process_constants::HEALTH_CHECK_INTERVAL,
            max_restart_attempts: process_constants::MAX_RESTART_ATTEMPTS,
            restart_delay: process_constants::RESTART_DELAY,
        }
    }
}
