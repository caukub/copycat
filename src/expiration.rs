use std::time::{Duration, SystemTime};

use tracing::{debug, trace};

use crate::configuration::StorageSettings;

const CHECK_PERIOD: Duration = Duration::from_hours(1);
const MAX_AGE: Duration = Duration::from_secs(3600 * 24 * 365);

pub async fn delete_expired(data: &StorageSettings) -> std::io::Result<()> {
    let mut interval = tokio::time::interval(CHECK_PERIOD);

    interval.tick().await;

    loop {
        let mut entries = tokio::fs::read_dir(data.directory.clone()).await?;
        let now = SystemTime::now();

        let mut deleted_count: u32 = 0;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file()
                && let Ok(metadata) = entry.metadata().await
                && let Ok(modified) = metadata.modified()
                && let Ok(elapsed) = now.duration_since(modified)
                && elapsed > MAX_AGE
            {
                tokio::fs::remove_file(&path).await?;
                tokio::fs::remove_file(data.directory.join("metadata")).await?;

                trace!("Deleted file {}", path.display());

                deleted_count += 1;
            }
        }

        debug!("Deleted {} expired pastes", deleted_count);

        interval.tick().await;
    }

    #[allow(unreachable_code)]
    Ok(())
}
