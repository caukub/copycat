use crate::paste::analyzer::PasteType;
use crate::paste::Paste;
use crate::routes::get::get_paste_type;
use crate::{AppError, AppState};
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use mclog::log::Log;

pub async fn get_raw(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Response, AppError> {
    let paste = Paste::new(id.clone(), app_state.configuration.clone());

    let paste_type = get_paste_type(id.clone(), &paste, app_state.clone()).await?;

    if let PasteType::Log = paste_type {
        let lines = paste.lines().await?;

        let log = Log::new(lines);

        let lines = log.first_n_lines_hideips(40_000).await;

        Ok(lines.join("\n").into_response())
    } else {
        let content =
            tokio::fs::read_to_string(app_state.configuration.storage.directory.join(id)).await?;

        Ok(content.into_response())
    }
}
