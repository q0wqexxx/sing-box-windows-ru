use crate::entity::config_model;
use crate::utils::app_util::get_work_dir;
use crate::utils::config_util::ConfigUtil;
use std::error::Error;
use std::path::Path;
use tracing::info;
use crate::app::constants::{paths, network, config as config_constants, messages};
use serde_json::{json, Value};
use reqwest::Client;
use tauri::{Runtime, Emitter};

// Изменение режима прокси на системный прокси
#[tauri::command]
pub fn set_system_proxy() -> Result<(), String> {
    let config_path = paths::get_config_path();
    let json_util =
        ConfigUtil::new(config_path.to_str().unwrap()).map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    let mut json_util = json_util;
    let target_keys = vec!["inbounds"];
    let new_structs = vec![config_model::Inbound {
        r#type: config_constants::DEFAULT_INBOUND_TYPE.to_string(),
        tag: config_constants::DEFAULT_INBOUND_TAG.to_string(),
        listen: Some(network::DEFAULT_LISTEN_ADDRESS.to_string()),
        listen_port: Some(network::DEFAULT_PROXY_PORT),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: None,
        set_system_proxy: Some(true),
    }];

    json_util.update_key(target_keys.clone(), serde_json::to_value(new_structs).unwrap());
    match json_util.save_to_file() {
        Ok(_) => {
            info!("{}", messages::INFO_PROXY_MODE_ENABLED);
            Ok(())
        }
        Err(e) => Err(format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e)),
    }
}

// Изменение режима TUN на прокси режим
#[tauri::command]
pub fn set_tun_proxy() -> Result<(), String> {
    set_tun_proxy_impl().map_err(|e| format!("Не удалось установить TUN прокси: {}", e))
}

fn set_tun_proxy_impl() -> Result<(), Box<dyn Error>> {
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let mut json_util = ConfigUtil::new(path.to_str().unwrap())?;

    let target_keys = vec!["inbounds"]; // Измените на ваш путь к свойству
    let new_structs = vec![
        config_model::Inbound {
            r#type: "mixed".to_string(),
            tag: "mixed-in".to_string(),
            listen: Some("0.0.0.0".to_string()),
            listen_port: Some(2080),
            address: None,
            auto_route: None,
            strict_route: None,
            stack: None,
            sniff: None,
            set_system_proxy: None,
        },
        config_model::Inbound {
            r#type: "tun".to_string(),
            tag: "tun-in".to_string(),
            listen: None,
            listen_port: None,
            address: Some(vec![
                "172.18.0.1/30".to_string(),
                "fdfe:dcba:9876::1/126".to_string(),
            ]),
            auto_route: Some(true),
            strict_route: Some(true),
            stack: Some("mixed".to_string()),
            sniff: None,
            set_system_proxy: None,
        },
    ];

    json_util.modify_property(
        &target_keys,
        serde_json::to_value(new_structs).map_err(|e| format!("Не удалось сериализовать конфигурацию: {}", e))?,
    );
    json_util
        .save()
        .map_err(|e| format!("Не удалось сохранить файл конфигурации: {}", e))?;

    info!("TUN прокси режим установлен");
    Ok(())
}

// Переключение версии IP
#[tauri::command]
pub fn toggle_ip_version(prefer_ipv6: bool) -> Result<(), String> {
    info!(
        "Начало переключения режима версии IP: {}",
        if prefer_ipv6 { "IPv6 предпочтительно" } else { "Только IPv4" }
    );

    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");

    // Чтение содержимого файла
    let content = std::fs::read_to_string(&path).map_err(|e| format!("Не удалось прочитать файл конфигурации: {}", e))?;

    // Прямая замена строки
    let modified_content = if prefer_ipv6 {
        content.replace("\"ipv4_only\"", "\"prefer_ipv6\"")
    } else {
        content.replace("\"prefer_ipv6\"", "\"ipv4_only\"")
    };

    // Проверка, является ли измененное содержимое допустимым JSON
    serde_json::from_str::<serde_json::Value>(&modified_content)
        .map_err(|e| format!("Измененная конфигурация не является допустимым JSON: {}", e))?;

    // Сохранение измененного содержимого
    std::fs::write(&path, modified_content).map_err(|e| format!("Не удалось сохранить файл конфигурации: {}", e))?;

    info!(
        "Режим версии IP успешно переключен на: {}",
        if prefer_ipv6 { "IPv6 предпочтительно" } else { "Только IPv4" }
    );
    Ok(())
}

// Получение API токена
#[tauri::command]
pub fn get_api_token() -> String {
    network::DEFAULT_API_TOKEN.to_string()
}

/// Получение списка прокси
#[tauri::command]
pub async fn get_proxies() -> Result<Value, String> {
    let token = get_api_token();
    let url = format!("http://{}:{}/proxies?token={}", 
        network::DEFAULT_CLASH_API_ADDRESS, 
        network::DEFAULT_CLASH_API_PORT,
        token);
    
    // Создание HTTP клиента без прокси
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("Не удалось создать HTTP клиент: {}", e))?;
    
    // Отправка запроса и получение ответа
    let response = client.get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Не удалось получить список прокси: {}", e))?;
    
    // Разбор ответа в JSON
    let json = response.json::<Value>()
        .await
        .map_err(|e| format!("Не удалось разобрать список прокси: {}", e))?;
    
    Ok(json)
}

/// Переключение прокси
#[tauri::command]
pub async fn change_proxy(group: String, proxy: String) -> Result<(), String> {
    let token = get_api_token();
    let url = format!("http://{}:{}/proxies/{}?token={}", 
        network::DEFAULT_CLASH_API_ADDRESS, 
        network::DEFAULT_CLASH_API_PORT, 
        group, 
        token);
    
    // Создание HTTP клиента без прокси
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("Не удалось создать HTTP клиент: {}", e))?;
    
    // Тело запроса
    let payload = json!({
        "name": proxy
    });
    
    // Отправка запроса и получение ответа
    let response = client.put(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Не удалось переключить прокси: {}", e))?;
    
    // Проверка статуса ответа
    if !response.status().is_success() {
        return Err(format!("Сервер вернул ошибку: {}", response.status()));
    }
    
    Ok(())
}

/// Тестирование задержки узла
#[tauri::command]
pub async fn test_node_delay(name: String) -> Result<u64, String> {
    // Использование URL по умолчанию для тестирования
    let test_url = "https://www.gstatic.com/generate_204";
    let token = get_api_token();
    let url = format!("http://{}:{}/proxies/{}/delay?url={}&timeout=5000&token={}", 
        network::DEFAULT_CLASH_API_ADDRESS, 
        network::DEFAULT_CLASH_API_PORT, 
        name, 
        urlencoding::encode(test_url),
        token);
    
    // Создание HTTP клиента без прокси
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("Не удалось создать HTTP клиент: {}", e))?;
    
    // Отправка запроса и получение ответа
    let response = client.get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Не удалось протестировать задержку узла: {}", e))?;
    
    // Разбор ответа в JSON
    let json = response.json::<Value>()
        .await
        .map_err(|e| format!("Не удалось разобрать результат теста: {}", e))?;
    
    // Получение значения задержки
    let delay = json["delay"]
        .as_u64()
        .unwrap_or(0);
    
    Ok(delay)
}

/// Пакетное тестирование задержки узлов
#[tauri::command]
pub async fn batch_test_nodes<R: Runtime>(
    window: tauri::Window<R>,
    nodes: Vec<String>, 
    server: Option<String>
) -> Result<(), String> {
    // Использование URL по умолчанию или указанного URL для тестирования
    let test_url = server.unwrap_or_else(|| "https://www.gstatic.com/generate_204".to_string());
    let token = get_api_token();
    
    // Создание HTTP клиента без прокси
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("Не удалось создать HTTP клиент: {}", e))?;
    
    // Перебор списка узлов для тестирования
    for (index, name) in nodes.iter().enumerate() {
        // Формирование URL запроса
        let url = format!("http://{}:{}/proxies/{}/delay?url={}&timeout=5000&token={}", 
            network::DEFAULT_CLASH_API_ADDRESS, 
            network::DEFAULT_CLASH_API_PORT,
            name,
            urlencoding::encode(&test_url),
            token);
        
        // Отправка события прогресса
        let _ = window.emit("test-nodes-progress", json!({
            "current": index + 1,
            "total": nodes.len(),
            "node": name,
            "status": "testing"
        }));
        
        // Отправка запроса и получение результата
        match client.get(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .send().await {
            Ok(response) => {
                match response.json::<Value>().await {
                    Ok(data) => {
                        // Отправка события результата теста
                        let _ = window.emit("test-node-result", json!({
                            "name": name,
                            "delay": data["delay"],
                            "success": true
                        }));
                    },
                    Err(e) => {
                        // Отправка события ошибки теста
                        let _ = window.emit("test-node-result", json!({
                            "name": name,
                            "delay": 0,
                            "success": false,
                            "error": format!("Не удалось разобрать результат: {}", e)
                        }));
                    }
                }
            },
            Err(e) => {
                // Отправка события ошибки теста
                let _ = window.emit("test-node-result", json!({
                    "name": name,
                    "delay": 0,
                    "success": false,
                    "error": format!("Запрос не удался: {}", e)
                }));
            }
        }
        
        // Короткая задержка, чтобы избежать слишком быстрого отправления запросов
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    // Отправка события завершения тестирования
    let _ = window.emit("test-nodes-complete", json!({
        "total": nodes.len()
    }));
    
    Ok(())
}

/// Получение информации о версии ядра
#[tauri::command]
pub async fn get_version_info() -> Result<Value, String> {
    let token = get_api_token();
    let url = format!("http://{}:{}/version?token={}", 
        network::DEFAULT_CLASH_API_ADDRESS, 
        network::DEFAULT_CLASH_API_PORT,
        token);
    
    // Создание HTTP клиента без прокси
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("Не удалось создать HTTP клиент: {}", e))?;
    
    // Отправка запроса и получение ответа
    let response = client.get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Не удалось получить информацию о версии: {}", e))?;
    
    // Разбор ответа в JSON
    let json = response.json::<Value>()
        .await
        .map_err(|e| format!("Не удалось разобрать информацию о версии: {}", e))?;
    
    Ok(json)
}

/// Получение списка правил
#[tauri::command]
pub async fn get_rules() -> Result<Value, String> {
    let token = get_api_token();
    let url = format!("http://{}:{}/rules?token={}", 
        network::DEFAULT_CLASH_API_ADDRESS, 
        network::DEFAULT_CLASH_API_PORT,
        token);
    
    // Создание HTTP клиента без прокси
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("Не удалось создать HTTP клиент: {}", e))?;

    info!("Получение списка правил {}", url);
    // Отправка запроса и получение ответа
    let response = client.get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Не удалось получить список правил: {}", e))?;
    
    // Разбор ответа в JSON
    let json = response.json::<Value>()
        .await
        .map_err(|e| format!("Не удалось разобрать список правил: {}", e))?;
    
    Ok(json)
}