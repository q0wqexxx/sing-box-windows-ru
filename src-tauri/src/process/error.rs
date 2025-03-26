use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ProcessError {
    #[error("Процесс уже запущен")]
    AlreadyRunning,

    #[error("Процесс не запущен")]
    NotRunning,

    #[error("Ошибка запуска: {0}")]
    StartFailed(String),

    #[error("Ошибка остановки: {0}")]
    StopFailed(String),

    #[error("Ошибка проверки состояния процесса: {0}")]
    StatusCheckFailed(String),

    #[error("Ошибка конфигурации: {0}")]
    ConfigError(String),

    #[error("Системная ошибка: {0}")]
    SystemError(String),

    #[error("Ошибка прав доступа: {0}")]
    PermissionError(String),

    #[error("Сетевая ошибка: {0}")]
    NetworkError(String),

    #[error("Неизвестная ошибка: {0}")]
    Unknown(String),
}

#[derive(Debug, Clone)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed(String),
}

impl fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessStatus::Starting => write!(f, "Запуск"),
            ProcessStatus::Running => write!(f, "Запущен"),
            ProcessStatus::Stopping => write!(f, "Остановка"),
            ProcessStatus::Stopped => write!(f, "Остановлен"),
            ProcessStatus::Failed(err) => write!(f, "Ошибка: {}", err),
        }
    }
}

pub type Result<T> = std::result::Result<T, ProcessError>;