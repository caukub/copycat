use crate::routes::api::ApiError;
use crate::CURRENT_DIRECTORY;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Response {
    markdown: Option<String>,
}

pub async fn get_frontend_api_detail(
    Path(detail_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let detail_path = CURRENT_DIRECTORY
        .join("details")
        .join(format!("{}.md", detail_id));
    let markdown = tokio::fs::read_to_string(detail_path.clone())
        .await
        .map_err(|_| ApiError::Client("Couldn't read detail into String"))?;

    let response = Response {
        markdown: Some(markdown),
    };

    Ok(Json(response).into_response())
}
