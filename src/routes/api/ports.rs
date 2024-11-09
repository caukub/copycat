use crate::routes::api::{get_analyzer_details, ApiError};
use crate::AppState;
use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    response::Response,
    Json,
};

pub async fn get_api_ports(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Response, ApiError> {
    let configuration = app_state.configuration;

    let info = get_analyzer_details(
        id,
        &configuration,
        0,
        configuration.analyzer.lines_limits.ports,
    )
    .await?;

    Ok(Json(info.ports).into_response())
}
