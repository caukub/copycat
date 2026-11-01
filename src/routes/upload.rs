use axum::{
    BoxError,
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
    response::Redirect,
};
use futures_util::{Stream, TryStreamExt};
use rand::{Rng, distr::Alphanumeric};
use serde::Serialize;
use std::{io, path::PathBuf, pin::pin};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
};
use tokio_util::io::StreamReader;
use tracing::trace;

use crate::AppState;

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
enum PasteType {
    Log,
    Yaml,
    Json,
    Other,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
struct PasteMetadata {
    #[serde(rename = "type")]
    paste_type: PasteType,
}

#[tracing::instrument(skip(multipart, state))]
pub async fn upload_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Redirect, (StatusCode, String)> {
    let id = generate_id(state.configuration.paste.id_length);

    // save max 5 MB, check if json/yaml only if < 1.5 MB

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

    let paste_content = tokio::fs::read_to_string(storage_configuration.directory.join(&id))
        .await
        .unwrap();

    let paste_type = get_paste_type(&paste_content).await;

    let metadata = PasteMetadata { paste_type };

    let json_metadata = serde_json::to_string(&metadata).unwrap();

    let mut file =
        tokio::fs::File::create(storage_configuration.directory.join("metadata").join(&id))
            .await
            .unwrap();
    file.write_all(json_metadata.as_bytes()).await.unwrap();

    Ok(Redirect::to(&id))
}

// check velikosti
// prvni musi byt json pak yaml
// mozna se na tohle uplne vykaslat kdyz se pouziva magika
// tokio spawn? asi ne omezime na 1 MB
async fn get_paste_type(content: &str) -> PasteType {

    // JSON needs to be first since every valid JSON is valid YAML
    if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
        return PasteType::Json;
    } else if serde_yaml::from_str::<serde_yaml::Value>(&content).is_ok() {
        return PasteType::Yaml;
    }

    PasteType::Other
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
