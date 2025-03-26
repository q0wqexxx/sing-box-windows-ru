use futures_util::StreamExt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tracing::{error, info};
use zip::ZipArchive;
use crate::app::constants::{messages, network};

// Загрузка файла по URL в указанное место
pub async fn download_file<F>(url: String, path: &str, progress_callback: F) -> Result<(), String>
where
    F: Fn(u32) + Send + 'static,
{
    let file_path = Path::new(path);
    info!("{}: {} -> {}", messages::INFO_DOWNLOAD_STARTED, url, file_path.to_str().unwrap());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(network::HTTP_TIMEOUT_SECONDS))
        .no_proxy() // Отключение прокси
        .build()
        .map_err(|e| format!("{}: {}", messages::ERR_HTTP_CLIENT_FAILED, e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_REQUEST_FAILED, e))?;

    if !response.status().is_success() {
        return Err(format!("{}: {}", messages::ERR_SERVER_ERROR, response.status()));
    }

    // Получение размера файла, если не существует, то 0
    let total_size = response.content_length().unwrap_or(0);
    let unknown_size = total_size == 0;
    
    if unknown_size {
        info!("Не удалось получить размер файла, будет использоваться потоковая загрузка");
    }

    // Создание директории
    if let Some(parent) = file_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e);
            return Err(format!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e));
        }
    }

    // Создание временного файла
    let temp_path = file_path.with_extension("tmp");
    let mut file = File::create(&temp_path).map_err(|e| format!("{}: {}", messages::ERR_CREATE_FILE_FAILED, e))?;

    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();
    let mut last_percent = 0u32;
    let mut last_progress_update = std::time::Instant::now();

    // Начало загрузки
    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("{}: {}", messages::ERR_REQUEST_FAILED, e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("{}: {}", messages::ERR_WRITE_FILE_FAILED, e))?;

        downloaded += chunk.len() as u64;
        
        // В зависимости от того, известен ли размер файла, используется разный способ расчета прогресса
        if unknown_size {
            // Для файлов неизвестного размера обновление прогресса каждые 1 МБ, используя загруженный размер в качестве индикатора прогресса
            // Чем больше загружено, тем медленнее растет прогресс, создавая у пользователя ощущение непрерывной загрузки
            let now = std::time::Instant::now();
            if now.duration_since(last_progress_update).as_millis() > 500 { // Обновление каждые 500 мс
                let percent = ((downloaded as f64 / 1_000_000.0).min(100.0)) as u32;
                if percent != last_percent {
                    last_percent = percent;
                    progress_callback(percent);
                    last_progress_update = now;
                }
            }
        } else {
            // Известный размер файла, нормальный расчет процента
            let percent = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
            if percent != last_percent {
                last_percent = percent;
                progress_callback(percent);
            }
        }
    }

    // Завершение загрузки, переименование временного файла
    std::fs::rename(&temp_path, &file_path)
        .map_err(|e| format!("{}: {}", messages::ERR_WRITE_FILE_FAILED, e))?;

    Ok(())
}

pub async fn unzip_file(path: &str, to: &str) -> Result<(), String> {
    info!("{}: {} -> {}", messages::INFO_UNZIP_STARTED, path, to);

    // Открытие ZIP файла
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            error!("{}: {}", messages::ERR_OPEN_FILE_FAILED, e);
            return Err(format!("{}: {}", messages::ERR_OPEN_FILE_FAILED, e));
        }
    };

    // Создание объекта ZipArchive
    let mut archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(e) => {
            error!("{}: {}", messages::ERR_READ_ARCHIVE_FAILED, e);
            return Err(format!("{}: {}", messages::ERR_READ_ARCHIVE_FAILED, e));
        }
    };

    // Убедиться, что целевая директория существует
    if let Err(e) = std::fs::create_dir_all(to) {
        error!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e);
        return Err(format!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e));
    }

    // Перебор всех элементов в ZIP файле
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("{}: {}", messages::ERR_EXTRACT_FILE_FAILED, e))?;

        // Получение имени файла и удаление ведущих путей
        let file_name = match Path::new(file.name()).file_name() {
            Some(name) => name,
            None => {
                error!("{}: {}", messages::ERR_INVALID_FILENAME, file.name());
                continue;
            }
        };

        let outpath = Path::new(to).join(file_name);
        info!("{}: {}", messages::INFO_EXTRACTING_FILE, outpath.display());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(|e| format!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e))?;
        } else {
            // Создание родительской директории файла
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| format!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e))?;
                }
            }
            
            // Создание файла и запись содержимого
            let mut outfile = File::create(&outpath).map_err(|e| format!("{}: {}", messages::ERR_CREATE_FILE_FAILED, e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("{}: {}", messages::ERR_WRITE_FILE_FAILED, e))?;
        }
    }

    Ok(())
}

// Загрузка через прокси, при неудаче попытка прямой загрузки
pub async fn download_with_fallback<F>(
    original_url: &str, 
    path: &str, 
    progress_callback: F
) -> Result<(), String>
where
    F: Fn(u32) + Send + Clone + 'static,
{
    // Сначала попытка загрузки через прокси https://gh-proxy.com/https://github.com/...
    let proxy_url = format!("https://gh-proxy.com/{}", original_url);
    info!("Попытка загрузки через прокси: {}", proxy_url);
    
    match download_file(proxy_url, path, progress_callback.clone()).await {
        Ok(_) => {
            info!("Загрузка через прокси успешна");
            Ok(())
        },
        Err(e) => {
            info!("Не удалось загрузить через прокси: {}, попытка прямой загрузки", e);
            // Если загрузка через прокси не удалась, попытка прямой загрузки
            download_file(original_url.to_string(), path, progress_callback).await
        }
    }
}
