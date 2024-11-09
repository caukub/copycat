use crate::configuration::Settings;
use crate::paste::Paste;
use axum::response::{IntoResponse, Response};
use axum::Json;
use mclog::analyzer::{Analyzer, DynamicAnalyzerDetails};
use mclog::log::Log;
use serde::Serialize;
use tracing::log::info;

pub mod all;
pub mod frontend;
pub mod leaks;
pub mod plugins;
pub mod ports;
pub mod upload;

#[derive(Debug)]
pub enum ApiError {
    Client(&'static str),
    Server(&'static str),
}

#[derive(Serialize)]
struct ApiResponse {
    error: bool,
    message: Option<&'static str>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let err = match self {
            ApiError::Client(err) => err,
            ApiError::Server(err) => err,
        };
        info!("API error occurred: '{}'", err);
        Json(ApiResponse {
            error: true,
            message: Some(err),
        })
        .into_response()
    }
}

async fn get_paste_lines(
    id: String,
    configuration: &Settings,
    limit: usize,
) -> Result<Vec<String>, ApiError> {
    let paste = Paste::new(id, configuration.clone());
    let lines = match paste.lines().await {
        Ok(lines) => lines,
        Err(_err) => return Err(ApiError::Server("Couldn't get paste lines")),
    };

    let log = Log::new(lines);
    let lines = log.first_n_lines_hideips(limit).await;

    Ok(lines)
}

async fn get_analyzer_details(
    id: String,
    configuration: &Settings,
    plugins_limit: usize,
    ports_limit: usize,
) -> Result<DynamicAnalyzerDetails, ApiError> {
    let lines =
        get_paste_lines(id, configuration, std::cmp::max(plugins_limit, ports_limit)).await?;

    Ok(Analyzer::new(&lines).build(plugins_limit, ports_limit))
}
