use crate::routes::post::upload_file;
use crate::{AppError, AppState};
use axum::{
    extract::{Multipart, State},
    response::Response,
};

pub async fn post_upload(
    State(app_state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    upload_file(app_state, multipart).await
}
