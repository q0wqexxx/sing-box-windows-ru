use crate::utils::app_util::get_work_dir;
use serde_json::json;
use std::os::windows::process::CommandExt;
use std::path::Path;
use tauri::Emitter;
use crate::app::constants::{api, messages};

// Добавление новой структуры для информации о версии
#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub download_url: String,
    pub has_update: bool,
}

// Проверка обновлений
#[tauri::command]
pub async fn check_update(current_version: String) -> Result<UpdateInfo, String> {
    let client = reqwest::Client::new();

    // Получение информации о последней версии
    let response = client
        .get(api::GITHUB_API_URL)
        .header("User-Agent", api::USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_GET_VERSION_FAILED, e))?;

    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_GET_VERSION_FAILED, e))?;

    // Получение последнего номера версии
    let tag_name = release["tag_name"]
        .as_str()
        .ok_or_else(|| format!("{}: Не удалось разобрать номер версии", messages::ERR_GET_VERSION_FAILED))
        .map(|v| v.trim_start_matches('v').to_string())?;
    

    // Получение ссылки для загрузки
    let assets = release["assets"].as_array().ok_or_else(|| {
        format!("{}: Не удалось получить ресурсы для загрузки", messages::ERR_GET_VERSION_FAILED)
    })?;

    // Поиск установщика для Windows
    let mut download_url = String::new();
    for asset in assets {
        let name = asset["name"].as_str().unwrap_or("");
        if name.ends_with(".msi") || name.ends_with(".exe") {
            download_url = asset["browser_download_url"]
                .as_str()
                .unwrap_or("")
                .to_string();
            break;
        }
    }

    if download_url.is_empty() {
        return Err(format!("{}: Не удалось получить ссылку для загрузки", messages::ERR_GET_VERSION_FAILED));
    }

    // Простое сравнение номеров версий
    let has_update = tag_name != current_version;

    Ok(UpdateInfo {
        latest_version: tag_name.to_string(),
        download_url,
        has_update,
    })
}

// Загрузка и установка обновления
#[tauri::command]
pub async fn download_and_install_update(
    window: tauri::Window,
    download_url: String,
) -> Result<(), String> {
    
    let work_dir = get_work_dir();
    let download_path = Path::new(&work_dir).join("update.exe");

    // Отправка события начала загрузки
    let _ = window.emit(
        "update-progress",
        json!({
            "status": "downloading",
            "progress": 0,
            "message": "Начало загрузки обновления..."
        }),
    );

    // Загрузка файла обновления
    let window_clone = window.clone();
    // Использование функции загрузки с резервным вариантом
    if let Err(e) = crate::utils::file_util::download_with_fallback(
        &download_url,
        download_path.to_str().unwrap(),
        move |progress| {
            let _ = window_clone.emit(
                "update-progress",
                json!({
                    "status": "downloading",
                    "progress": progress,
                    "message": format!("Загрузка: {}%", progress)
                }),
            );
        },
    ).await {
        return Err(format!("Не удалось загрузить обновление: {}", e));
    }

    // Отправка события завершения загрузки
    let _ = window.emit(
        "update-progress",
        json!({
            "status": "completed",
            "progress": 100,
            "message": "Загрузка завершена, подготовка к установке..."
        }),
    );

    // Запуск установщика
    std::process::Command::new(download_path)
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("Не удалось запустить установщик: {}", e))?;

    Ok(())
}