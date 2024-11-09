use crate::routes::api::{get_analyzer_details, ApiError};
use crate::AppState;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};

pub async fn get_api_all(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Response, ApiError> {
    let lines_limits = app_state.configuration.analyzer.lines_limits.clone();
    let info = get_analyzer_details(
        id,
        &app_state.configuration,
        lines_limits.plugins,
        lines_limits.ports,
    )
    .await?;

    Ok(Json(info).into_response())
}
