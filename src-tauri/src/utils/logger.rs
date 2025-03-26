use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    Layer,
    filter::LevelFilter,
    EnvFilter, Registry,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::app::constants::log;

use crate::config::LogConfig;

// Статический флаг для обеспечения однократной инициализации
static LOGGER_INITIALIZED: AtomicBool = AtomicBool::new(false);
static INIT_ONCE: Once = Once::new();

// Статическое хранилище для всех WorkerGuard, чтобы гарантировать их сохранение на протяжении всего жизненного цикла программы
static GLOBAL_GUARDS: Lazy<Mutex<Vec<WorkerGuard>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub struct Logger;

impl Logger {
    // Инициализация системы логирования
    pub fn init(config: &LogConfig) -> Self {
        // Использование атомарных операций и Once для обеспечения однократной инициализации
        let already_initialized = LOGGER_INITIALIZED.load(Ordering::Relaxed);
        if already_initialized {
            return Self;
        }
        
        // Использование Once для обеспечения однократного выполнения кода инициализации
        INIT_ONCE.call_once(|| {
            // Установка атомарного флага
            LOGGER_INITIALIZED.store(true, Ordering::Relaxed);
            
            // Убедиться, что каталог для логов существует
            std::fs::create_dir_all(&config.dir).expect("Не удалось создать каталог для логов");

            // Установка уровня логирования по умолчанию
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", &config.level);
            }

            // Получение настройки уровня логирования
            let env_filter_string = std::env::var("RUST_LOG").unwrap_or_else(|_| config.level.clone());

            // Конфигурация ротации файлов логов
            let rotation = match config.rotation.as_str() {
                log::rotation::HOURLY => Rotation::HOURLY,
                log::rotation::DAILY => Rotation::DAILY,
                log::rotation::NEVER => Rotation::NEVER,
                _ => Rotation::DAILY, // По умолчанию ежедневная ротация
            };
            
            // Создание временного контейнера для guard
            let mut guards = Vec::new();

            // Создание глобального файла логов для записи всех уровней логов
            let all_appender = RollingFileAppender::builder()
                .rotation(rotation.clone())
                .filename_prefix(&format!("{}-all", &config.file_name_prefix))
                .max_log_files(config.max_files as usize)
                .build(&config.dir)
                .expect("Не удалось создать глобальный appender для логов");
            
            // Конфигурация более короткого интервала обновления, чтобы логи быстрее записывались в файл
            let (all_non_blocking, all_guard) = tracing_appender::non_blocking(all_appender);
            guards.push(all_guard);

            // Создание файла логов для ошибок
            let error_appender = RollingFileAppender::builder()
                .rotation(rotation.clone())
                .filename_prefix(&format!("{}-error", &config.file_name_prefix))
                .max_log_files(config.max_files as usize)
                .build(&config.dir)
                .expect("Не удалось создать appender для логов ошибок");
            
            let (error_non_blocking, error_guard) = tracing_appender::non_blocking(error_appender);
            guards.push(error_guard);

            // Создание файла логов для предупреждений
            let warn_appender = RollingFileAppender::builder()
                .rotation(rotation.clone())
                .filename_prefix(&format!("{}-warn", &config.file_name_prefix))
                .max_log_files(config.max_files as usize)
                .build(&config.dir)
                .expect("Не удалось создать appender для логов предупреждений");
            
            let (warn_non_blocking, warn_guard) = tracing_appender::non_blocking(warn_appender);
            guards.push(warn_guard);

            // Создание файла логов для информации
            let info_appender = RollingFileAppender::builder()
                .rotation(rotation)
                .filename_prefix(&format!("{}-info", &config.file_name_prefix))
                .max_log_files(config.max_files as usize)
                .build(&config.dir)
                .expect("Не удалось создать appender для логов информации");
            
            let (info_non_blocking, info_guard) = tracing_appender::non_blocking(info_appender);
            guards.push(info_guard);

            // Создание глобального подписчика
            let registry = Registry::default()
                // Вывод в консоль
                .with(
                    fmt::layer()
                        .with_ansi(true)
                        .with_line_number(true)
                        .with_thread_ids(true)
                        .with_target(true)
                        .with_writer(std::io::stdout)
                        .with_filter(EnvFilter::new(&env_filter_string))
                )
                // Глобальный файл логов - включает все уровни
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .with_line_number(true)
                        .with_thread_ids(true)
                        .with_target(true)
                        .with_file(true)
                        .with_writer(all_non_blocking)
                        .with_filter(EnvFilter::new(&env_filter_string))
                )
                // Файл логов ошибок - только уровень ERROR
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .with_line_number(true)
                        .with_thread_ids(true)
                        .with_target(true)
                        .with_file(true)
                        .with_writer(error_non_blocking)
                        .with_filter(LevelFilter::ERROR)
                )
                // Файл логов предупреждений - только уровень WARN
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .with_line_number(true)
                        .with_thread_ids(true)
                        .with_target(true)
                        .with_file(true)
                        .with_writer(warn_non_blocking)
                        .with_filter(LevelFilter::WARN)
                )
                // Файл логов информации - только уровень INFO
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .with_line_number(true)
                        .with_thread_ids(true)
                        .with_target(true)
                        .with_file(true)
                        .with_writer(info_non_blocking)
                        .with_filter(LevelFilter::INFO)
                );

            // Установка глобального подписчика по умолчанию
            tracing::subscriber::set_global_default(registry)
                .expect("Не удалось установить глобальный подписчик логов");

            // Перемещение guard в глобальное статическое хранилище
            if let Ok(mut global_guards) = GLOBAL_GUARDS.lock() {
                global_guards.extend(guards);
            } else {
                eprintln!("Предупреждение: Не удалось заблокировать глобальные guards логгера, логи могут не записываться корректно в файл");
            }
        });

        Self
    }
}

// Предоставление удобного метода инициализации
pub fn init_logger() -> Logger {
    Logger::init(&LogConfig::default())
}
