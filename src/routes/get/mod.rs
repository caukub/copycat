use crate::paste::analyzer::PasteType;
use crate::paste::Paste;
use crate::{AppError, AppState};
use anyhow::anyhow;
use fred::interfaces::KeysInterface;

pub mod raw;

pub async fn get_paste_type(
    id: String,
    paste: &Paste,
    app_state: AppState,
) -> Result<PasteType, AppError> {
    let paste_type: Option<String> = app_state
        .redis_state
        .pool
        .get(id.clone())
        .await
        .unwrap_or(None);

    let paste_type = match paste_type {
        Some(pt) => PasteType::from(pt),
        None => {
            if paste.file_path().exists() {
                tokio::fs::remove_file(paste.file_path()).await?;
                return Err(AppError(anyhow!("Couldn't remove {id}")));
            } else {
                PasteType::Other
            }
        }
    };

    Ok(paste_type)
}
