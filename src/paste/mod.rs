use crate::configuration::Settings;
use crate::CURRENT_DIRECTORY;
use std::path::PathBuf;
use tokio::io::AsyncBufReadExt;

pub mod analyzer;

pub struct Paste {
    id: String,
    configuration: Settings,
}

impl Paste {
    pub fn new(id: String, configuration: Settings) -> Self {
        Self { id, configuration }
    }

    pub fn data_directory(&self) -> PathBuf {
        CURRENT_DIRECTORY.join(&self.configuration.storage.directory)
    }

    pub fn file_path(&self) -> PathBuf {
        self.data_directory().join(&self.id)
    }

    pub async fn file(&self) -> tokio::io::Result<tokio::fs::File> {
        tokio::fs::File::open(self.file_path()).await
    }

    pub async fn lines(
        &self,
    ) -> tokio::io::Result<tokio::io::Lines<tokio::io::BufReader<tokio::fs::File>>> {
        let file = self.file().await;

        let reader = tokio::io::BufReader::new(file?);

        let lines = reader.lines();

        Ok(lines)
    }
}
