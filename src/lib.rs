pub mod configuration;
pub mod middleware;
pub mod paste;
pub mod redis;
pub mod routes;

use crate::configuration::Settings;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use fred::clients::RedisPool;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::log::error;

static CURRENT_DIRECTORY: LazyLock<PathBuf> = LazyLock::new(|| match std::env::current_dir() {
    Ok(current_dir) => current_dir,
    Err(err) => panic!("Failed to get current directory: '{}'", err),
});

#[derive(Clone)]
pub struct AppState {
    pub configuration: Settings,
    pub redis_state: RedisState,
}

#[derive(Clone)]
pub struct RedisState {
    pub pool: RedisPool,
}

impl FromRef<AppState> for RedisState {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.redis_state.clone()
    }
}

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Something went wrong: {}", self.0);

        (StatusCode::INTERNAL_SERVER_ERROR, Redirect::to("/")).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
