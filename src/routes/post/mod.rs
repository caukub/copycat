use crate::configuration::Settings;
use crate::paste::analyzer::PasteAnalyzer;
use crate::paste::Paste;
use crate::{AppError, AppState};
use axum::body::Bytes;
use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::BoxError;
use fred::interfaces::KeysInterface;
use fred::types::Expiration;
use futures::{Stream, TryStreamExt};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::io;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufWriter};
use tokio_util::io::StreamReader;
use tracing::log::{error, warn};

pub mod upload;

async fn stream_to_file<S, E>(id: String, stream: S, configuration: &Settings)
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    async {
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader =
            StreamReader::new(body_with_io_error).take(configuration.paste.size_limit as u64);
        futures::pin_mut!(body_reader);

        let path = std::path::Path::new(&configuration.storage.directory).join(id);
        let mut file = BufWriter::new(File::create(path).await.unwrap());

        if let Err(err) = tokio::io::copy(&mut body_reader, &mut file).await {
            error!("Streaming to file failed: {err}");
        }
    }
    .await
}

#[allow(dependency_on_unit_never_type_fallback)]
async fn upload_file(app_state: AppState, mut multipart: Multipart) -> Result<Response, AppError> {
    let id = gen_id(app_state.configuration.storage.id_length);

    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some("content") = field.name() {
            stream_to_file(id.clone(), field, &app_state.configuration).await;
        }
    }

    let paste = Paste::new(id.clone(), app_state.configuration.clone());

    let content = tokio::fs::read_to_string(paste.file_path()).await?;

    if content.is_empty() {
        tokio::fs::remove_file(paste.file_path()).await?;
        return Ok((StatusCode::BAD_REQUEST, Redirect::to("/")).into_response());
    }

    let paste_analyzer = PasteAnalyzer::new();
    let paste_type = paste_analyzer.paste_type(content.as_bytes());

    app_state
        .redis_state
        .pool
        .set(
            id.clone(),
            paste_type,
            get_expiration(&app_state.configuration),
            None,
            false,
        )
        .await?;

    Ok(Redirect::to(&format!("/{}", id)).into_response())
}

pub fn get_expiration(configuration: &Settings) -> Option<Expiration> {
    let expiration_in_hours = configuration.storage.expiration_in_hours;

    if expiration_in_hours > 0.0 {
        let expiration_in_seconds = (expiration_in_hours * 3600.0) as i64;
        Some(Expiration::EX(expiration_in_seconds))
    } else {
        if expiration_in_hours != 0.0 {
            warn!("Expiration is set to '{}' but should be set to positive number or 0 (disabled).\nExpiration is disabled.", expiration_in_hours);
        }
        None
    }
}

pub fn gen_id(length: u16) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect()
}
