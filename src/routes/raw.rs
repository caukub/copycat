use axum::{
    body::Body,
    extract::{Path, State},
    http::header::CONTENT_TYPE,
    response::{IntoResponse, Redirect, Response},
};
use tokio_util::io::ReaderStream;
use tracing::{debug, error};

use crate::AppState;

#[tracing::instrument(skip(state))]
pub async fn raw_handler(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    let file_path = state.configuration.storage.directory.join(&id);

    let file = match tokio::fs::File::open(file_path).await {
        Ok(file) => file,
        Err(err) => {
            debug!("Failed to open '{}': {}", id, err);
            return Redirect::to("/").into_response();
        }
    };

    let stream = ReaderStream::new(file);

    Response::builder()
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header("X-Content-Type-Options", "nosniff")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|err| {
            error!("Failed to build the Response: {}", err);
            Redirect::to("/").into_response()
        })
}
