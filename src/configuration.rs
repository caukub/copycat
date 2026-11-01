use std::path::PathBuf;

use serde::Deserialize;
use tracing::error;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub storage: StorageSettings,
    pub paste: PasteSettings,
    pub parser: ParserSettings,
    pub analyzer: AnalyzerSettings,
    pub cors: CorsSettings,
    pub api: ApiSettings,
}

#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
    #[serde(rename = "body_limit_in_bytes")]
    pub body_limit: usize,
}

#[derive(Deserialize, Clone)]
pub struct StorageSettings {
    pub directory: PathBuf,
    #[serde(rename = "max_size_in_bytes")]
    pub max_size: usize,
}

#[derive(Deserialize, Clone)]
pub struct ParserSettings {
    pub level_suffix_lines_limit: usize,
    pub custom_highlighting_delimiters: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct AnalyzerSettings {
    pub limits: String,
}

#[derive(Deserialize, Clone)]
pub struct CorsSettings {
    pub allow_origin: String,
}

#[derive(Deserialize, Clone)]
pub struct ApiSettings {}

#[derive(Deserialize, Clone)]
pub struct PasteSettings {
    pub id_length: usize,
    pub expiration_in_hours: u32,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().unwrap_or_else(|err| {
        error!("Failed to get current directory: {}", err);
        std::process::exit(1);
    });

    let configuration_directory = base_path.join("configuration");

    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("config.toml"),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}
