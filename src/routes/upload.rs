use axum::{
    BoxError,
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
    response::Redirect,
};
use futures_util::{Stream, TryStreamExt};
use magika::ContentType;
use rand::{Rng, distr::Alphanumeric};
use serde::Serialize;
use std::{io, path::PathBuf, pin::pin, str::Lines, sync::LazyLock};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    sync::Mutex,
};
use tokio_util::io::StreamReader;
use tracing::{error, trace};

use crate::AppState;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
enum PasteType {
    Log,
    Yaml,
    Json,
    InvalidYaml,
    InvalidJson,
    Other,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
struct PasteMetadata {
    #[serde(rename = "type")]
    paste_type: PasteType,
}

pub static MAGIKA: LazyLock<Mutex<magika::Session>> =
    LazyLock::new(|| Mutex::new(magika::Session::new().unwrap()));

#[tracing::instrument(skip(multipart, state))]
pub async fn upload_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Redirect, (StatusCode, String)> {
    let id = generate_id(state.configuration.paste.id_length);

    let storage_configuration = state.configuration.storage;

    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(name) = field.name()
            && name == "content"
        {
            stream_to_file(
                &id,
                field,
                &storage_configuration.directory,
                storage_configuration.max_size,
            )
            .await?;
            trace!("Created paste '{}'", id);
        }
    }

    let bytes = tokio::fs::read(storage_configuration.directory.join(&id))
        .await
        .unwrap();

    let content = String::from_utf8_lossy(&bytes);

    let paste_type = get_paste_type(&bytes, &content).await;

    error!("{:?}", paste_type);

    let metadata = PasteMetadata { paste_type };

    let json_metadata = serde_json::to_string(&metadata).unwrap();

    let mut file =
        tokio::fs::File::create(storage_configuration.directory.join("metadata").join(&id))
            .await
            .unwrap();

    file.write_all(json_metadata.as_bytes()).await.unwrap();

    Ok(Redirect::to(&id))
}

async fn get_paste_type(bytes: &Vec<u8>, content: &str) -> PasteType {
    /// Content with size above this value (in bytes) will be checked only for the `Log` type
    const CHECK_ONLY_LOG_TYPE_SIZE: usize = 1_500_000;

    if is_log(content) {
        return PasteType::Log;
    }

    let mut is_valid_yaml = false;

    if bytes.len() < CHECK_ONLY_LOG_TYPE_SIZE {
        // JSON needs to be checked first since every valid JSON is valid YAML
        if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
            return PasteType::Json;
        } else if serde_yaml::from_str::<serde_yaml::Value>(&content).is_ok() {
            is_valid_yaml = true;
        }

        let file_type = match MAGIKA
            .lock()
            .await
            .identify_content_async(bytes.as_slice())
            .await
        {
            Ok(ft) => ft,
            Err(_) => return PasteType::Other,
        };

        match file_type.content_type() {
            Some(ContentType::Yaml) => {
                // often, raw text is valid YAML too
                if is_valid_yaml {
                    return PasteType::Yaml;
                } else {
                    return PasteType::InvalidYaml;
                }
            }
            Some(ContentType::Json) => return PasteType::InvalidJson,
            _ => {}
        }
    }

    PasteType::Other
}

fn is_log(content: &str) -> bool {
    const LOG_LEVELS: [&str; 7] = [
        "INFO", "WARN", "WARNING", "DEBUG", "SEVERE", "FATAL", "ERROR",
    ];

    let lines = content.lines();
    let line_count = lines.clone().count();

    let log_level_count = lines
        .filter(|line| LOG_LEVELS.iter().any(|lvl| line.contains(lvl)))
        .count();

    const THRESHOLDS: [(usize, usize); 5] = [(5, 1), (15, 2), (30, 4), (50, 6), (100, 8)];

    let required = THRESHOLDS
        .iter()
        .find(|&&(max_lines, _)| line_count < max_lines)
        .map_or(10, |&(_, min_levels)| min_levels);

    return log_level_count >= required;
}

async fn stream_to_file<S, E>(
    path: &str,
    stream: S,
    upload_directory: &PathBuf,
    file_limit: usize,
) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    async {
        let body_with_io_error = stream.map_err(io::Error::other);
        let mut body_reader = pin!(StreamReader::new(body_with_io_error).take(file_limit as u64));

        let path = std::path::Path::new(upload_directory).join(path);
        let mut file = BufWriter::new(File::create(path).await?);

        tokio::io::copy(&mut body_reader, &mut file).await?;

        file.flush().await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

fn generate_id(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
