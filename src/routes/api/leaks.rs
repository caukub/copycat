use crate::routes::api::{get_paste_lines, ApiError};
use crate::AppState;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use mclog::analyzer::static_analyzer::StaticAnalyzer;

pub async fn get_api_leaks(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Response, ApiError> {
    let configuration = app_state.configuration;
    let lines = get_paste_lines(
        id,
        &configuration,
        configuration.analyzer.lines_limits.plugins,
    )
    .await?;

    let mut suspicious_lines = Vec::new();

    for line in lines {
        if let Some(line) = StaticAnalyzer::leaked_plugin(&line) {
            suspicious_lines.push(line);
        }
    }

    Ok(Json(suspicious_lines).into_response())
}
