use crate::routes::api::{get_analyzer_details, ApiError};
use crate::AppState;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};

pub async fn get_api_plugins(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Response, ApiError> {
    let configuration = app_state.configuration;

    let info = get_analyzer_details(
        id,
        &configuration,
        configuration.analyzer.lines_limits.plugins,
        0,
    )
    .await?;

    Ok(Json(info.plugins).into_response())
}
