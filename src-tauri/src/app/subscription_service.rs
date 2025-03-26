use crate::entity::config_model::{CacheFileConfig, ClashApiConfig, Config};
use crate::app::constants::{paths, messages};
use crate::utils::config_util::ConfigUtil;
use crate::utils::app_util::get_work_dir;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tracing::{info, error};
use base64;

// Загрузка подписки
#[tauri::command]
pub async fn download_subscription(url: String) -> Result<(), String> {
    download_and_process_subscription(url)
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_SUBSCRIPTION_FAILED, e))?;
    let _ = crate::app::proxy_service::set_system_proxy();
    Ok(())
}

// Ручное добавление содержимого подписки
#[tauri::command]
pub async fn add_manual_subscription(content: String) -> Result<(), String> {
    process_subscription_content(content)
        .map_err(|e| format!("{}: {}", messages::ERR_PROCESS_SUBSCRIPTION_FAILED, e))?;
    let _ = crate::app::proxy_service::set_system_proxy();
    Ok(())
}

// Получение текущего содержимого конфигурационного файла
#[tauri::command]
pub fn get_current_config() -> Result<String, String> {
    let config_path = paths::get_config_path();
    
    // Проверка существования файла
    if !config_path.exists() {
        return Err(messages::ERR_CONFIG_READ_FAILED.to_string());
    }
    
    // Чтение содержимого файла
    match std::fs::read_to_string(config_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e)),
    }
}

// Переключение режима прокси (global, rule или tun)
#[tauri::command]
pub fn toggle_proxy_mode(mode: String) -> Result<String, String> {
    // Проверка параметра режима
    if !["global", "rule", "tun"].contains(&mode.as_str()) {
        return Err(format!("Недопустимый режим прокси: {}", mode));
    }
    
    info!("Переключение режима прокси на: {}", mode);
    
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    
    // Проверка существования файла
    if !path.exists() {
        return Err("Конфигурационный файл не существует, сначала добавьте подписку".to_string());
    }
    
    // Изменение конфигурационного файла
    match modify_default_mode(&path, mode.clone()) {
        Ok(_) => {
            info!("Режим прокси переключен на: {}", mode);
            Ok(format!("Режим прокси переключен на: {}", mode))
        },
        Err(e) => {
            error!("Не удалось переключить режим прокси: {}", e);
            Err(format!("Не удалось переключить режим прокси: {}", e))
        }
    }
}

// Изменение default_mode в конфигурационном файле
fn modify_default_mode(config_path: &Path, mode: String) -> Result<(), Box<dyn Error>> {
    // Чтение существующей конфигурации
    let mut json_util = ConfigUtil::new(config_path.to_str().unwrap())?;
    
    // Мы не используем метод get_value, так как он не существует
    // Вместо этого создаем новую конфигурацию и изменяем ее
    let target_keys = vec!["experimental"];
    
    // Создание новой конфигурации, установка mode
    let config = Config {
        clash_api: ClashApiConfig {
            external_controller: "127.0.0.1:12081".to_string(),
            external_ui: "metacubexd".to_string(),
            external_ui_download_url: "".to_string(),
            external_ui_download_detour: "Ручное переключение".to_string(),
            default_mode: mode, // Установка переданного режима
        },
        cache_file: CacheFileConfig { enabled: true },
    };
    
    // Обновление конфигурации
    json_util.modify_property(&target_keys, serde_json::to_value(config)?);
    json_util.save()?;
    
    Ok(())
}

async fn download_and_process_subscription(url: String) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    let user_agent = reqwest::header::HeaderValue::from_static("sing-box-windows/1.0 (sing-box; compatible; Windows NT 10.0)");
    headers.insert(reqwest::header::USER_AGENT, user_agent);

    let response = client.get(url).headers(headers).send().await?;
    let response_text = response.text().await?;

    // Проверка, является ли содержимое base64-кодированным, и декодирование при необходимости
    let text = if is_base64_encoded(&response_text) {
        info!("Обнаружено base64-кодированное содержимое, декодирование...");
        let decoded = match base64::decode(&response_text.trim()) {
            Ok(data) => data,
            Err(_) => {
                // Если стандартное декодирование не удалось, попытка URL-безопасного варианта base64
                base64::decode_config(&response_text.trim(), base64::URL_SAFE)
                    .map_err(|e| format!("Ошибка декодирования Base64: {}", e))?
            }
        };
        
        // Попытка преобразования декодированного содержимого в допустимую строку UTF-8
        match String::from_utf8(decoded.clone()) {
            Ok(s) => {
                // Проверка, является ли декодированное содержимое допустимым JSON или конфигурационным форматом
                if s.trim_start().starts_with('{') || s.contains("proxies:") {
                    s // Возвращение декодированного текста
                } else {
                    // Декодированное содержимое не похоже на допустимую конфигурацию, возможно, это ошибка, использование исходного текста
                    info!("Декодированное содержимое не является допустимым конфигурационным форматом, использование исходного содержимого");
                    response_text
                }
            },
            Err(_) => {
                // Если это не допустимая строка UTF-8, возвращение исходного текста
                info!("Декодированное содержимое не является допустимой строкой UTF-8, использование исходного содержимого");
                response_text
            }
        }
    } else {
        response_text
    };

    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let mut file = File::create(path.to_str().unwrap())?;
    file.write_all(text.as_bytes())?;

    let mut json_util = ConfigUtil::new(path.to_str().unwrap())?;
    let target_keys = vec!["experimental"];
    let config = Config {
        clash_api: ClashApiConfig {
            external_controller: "127.0.0.1:12081".to_string(),
            external_ui: "".to_string(),
            external_ui_download_url: "".to_string(),
            external_ui_download_detour: "".to_string(),
            default_mode: "rule".to_string(),
        },
        cache_file: CacheFileConfig { enabled: true },
    };
    json_util.modify_property(&target_keys, serde_json::to_value(config)?);
    json_util.save()?;

    info!("Подписка обновлена");
    Ok(())
}

// Обработка содержимого подписки (независимо от того, получено ли оно из URL или добавлено вручную)
fn process_subscription_content(content: String) -> Result<(), Box<dyn Error>> {
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let mut file = File::create(path.to_str().unwrap())?;
    file.write_all(content.as_bytes())?;

    let mut json_util = ConfigUtil::new(path.to_str().unwrap())?;
    let target_keys = vec!["experimental"];
    let config = Config {
        clash_api: ClashApiConfig {
            external_controller: "127.0.0.1:12081".to_string(),
            external_ui: "metacubexd".to_string(),
            external_ui_download_url: "".to_string(),
            external_ui_download_detour: "Ручное переключение".to_string(),
            default_mode: "rule".to_string(),
        },
        cache_file: CacheFileConfig { enabled: true },
    };
    json_util.modify_property(&target_keys, serde_json::to_value(config)?);
    json_util.save()?;

    info!("Подписка обновлена");
    Ok(())
}

// Улучшенная логика проверки base64
fn is_base64_encoded(text: &str) -> bool {
    // Сначала выполняется базовая проверка набора символов
    let is_valid_charset = text.trim().chars().all(|c| 
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || 
        c == '-' || c == '_' // Поддержка URL-безопасного варианта
    );
    
    if !is_valid_charset {
        return false;
    }
    
    let trimmed = text.trim();
    
    // Проверка длины (стандартная длина base64 должна быть кратна 4, возможно наличие завершающих '=')
    if trimmed.len() % 4 != 0 && !trimmed.ends_with('=') {
        return false;
    }
    
    // Избегание ложных срабатываний для обычного текста
    if trimmed.len() < 8 || trimmed.contains(" ") {
        return false;
    }
    
    // Попытка декодирования для проверки успешности (более точный, но менее производительный метод)
    let standard_decode_ok = base64::decode(trimmed).is_ok();
    let url_safe_decode_ok = base64::decode_config(trimmed, base64::URL_SAFE).is_ok();
    
    // Если декодирование успешно, проверка, является ли декодированное содержимое разумным (избегание ложных срабатываний)
    if standard_decode_ok || url_safe_decode_ok {
        // Проверка на наличие общих признаков формата подписки
        if trimmed.starts_with("ey") || trimmed.starts_with("dm") {
            return true; // Общие признаки начала base64-кодированного JSON или YAML
        }
    }
    
    false
}

// Получение текущего режима прокси
#[tauri::command]
pub fn get_current_proxy_mode() -> Result<String, String> {
    info!("Получение текущего режима прокси");
    
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    
    // Проверка существования конфигурационного файла
    if !path.exists() {
        return Ok("rule".to_string()); // По умолчанию возвращается режим rule
    }
    
    // Чтение конфигурационного файла
    match read_proxy_mode_from_config(&path) {
        Ok(mode) => {
            info!("Текущий режим прокси: {}", mode);
            Ok(mode)
        },
        Err(e) => {
            error!("Не удалось получить режим прокси: {}", e);
            Ok("rule".to_string()) // В случае ошибки по умолчанию возвращается режим rule
        }
    }
}

// Чтение режима прокси из конфигурационного файла
fn read_proxy_mode_from_config(config_path: &Path) -> Result<String, Box<dyn Error>> {
    // Чтение конфигурационного файла
    let mut file = File::open(config_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    // Разбор JSON
    let json: serde_json::Value = serde_json::from_str(&content)?;
    
    // Попытка чтения experimental.clash_api.default_mode
    if let Some(experimental) = json.get("experimental") {
        if let Some(clash_api) = experimental.get("clash_api") {
            if let Some(default_mode) = clash_api.get("default_mode") {
                if let Some(mode) = default_mode.as_str() {
                    return Ok(mode.to_string());
                }
            }
        }
    }
    
    // Если не найдено, возвращается режим rule по умолчанию
    Ok("rule".to_string())
}