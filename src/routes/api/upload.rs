use crate::paste::analyzer::PasteAnalyzer;
use crate::routes::api::ApiError;
use crate::routes::post::{gen_id, get_expiration};
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use fred::interfaces::KeysInterface;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    id: String,
}

pub async fn post_api_upload(
    State(app_state): State<AppState>,
    Json(request): Json<Request>,
) -> Result<axum::response::Response, ApiError> {
    let id = gen_id(app_state.configuration.storage.id_length);
    let paste_dir = app_state.configuration.storage.directory.join(id.clone());

    let mut file = tokio::fs::File::create(paste_dir).await.unwrap();
    file.write_all(request.content.as_bytes()).await.unwrap();

    let paste_analyzer = PasteAnalyzer::new();
    let paste_type = paste_analyzer.paste_type(request.content.as_bytes());

    let _: () = app_state
        .redis_state
        .pool
        .set(
            id.clone(),
            paste_type,
            get_expiration(&app_state.configuration),
            None,
            false,
        )
        .await
        .unwrap();

    Ok((StatusCode::OK, Json(Response { id })).into_response())
}
