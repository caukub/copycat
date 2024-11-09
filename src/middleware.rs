use crate::routes::api::ApiError;
use crate::AppState;
use axum::body::Body;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub async fn api_admin_middleware(
    State(app_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    if app_state.configuration.api.no_auth {
        Ok(next.run(request).await)
    } else {
        reject_unauthorized_request(request, next).await
    }
}

pub async fn api_middleware(
    State(app_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    if app_state.configuration.api.no_auth || app_state.configuration.api.public {
        Ok(next.run(request).await)
    } else {
        reject_unauthorized_request(request, next).await
    }
}

async fn reject_unauthorized_request(
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    if let Some(request_api_key) = request.headers().get("x-api-key") {
        if let Ok(required_api_key) = std::env::var("APP_API_KEY") {
            let request_api_key = match request_api_key.to_str() {
                Ok(key) => key,
                Err(_) => return reject("API key couldn't be converted into str"),
            };
            if request_api_key == required_api_key.as_str() {
                Ok(next.run(request).await)
            } else {
                reject("Wrong API key")
            }
        } else {
            reject("'APP_API_KEY' is not set")
        }
    } else {
        reject("No 'X-API-KEY' header found. Authorization is required.")
    }
}

fn reject(message: &str) -> Result<Response, ApiError> {
    Ok((
        StatusCode::UNAUTHORIZED,
        Json(json!({"error": true, "message": message})),
    )
        .into_response())
}
