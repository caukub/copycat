use crate::CURRENT_DIRECTORY;
use config::{Environment, Value};
use serde::Deserialize;
use std::path::PathBuf;

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let configuration_directory = CURRENT_DIRECTORY.join("configuration");

    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("config.toml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join("ports.toml"),
        ))
        .add_source(
            Environment::with_prefix("APP")
                .try_parsing(true)
                .separator("_")
                .list_separator(" "),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub application: Application,
    pub cors: Cors,
    pub paste: Paste,
    pub storage: Storage,
    pub analyzer: Analyzer,
    pub ports: Ports,
    pub api: Api,
    pub redis: Redis,
}

#[derive(Deserialize, Clone)]
pub struct Application {
    pub host: String,
    pub port: u16,
    #[serde(rename = "body_limit_in_bytes")]
    pub body_limit: usize,
}

#[derive(Deserialize, Clone)]
pub struct Cors {
    pub allow_origin: String,
}

#[derive(Deserialize, Clone)]
pub struct Paste {
    #[serde(rename = "size_limit_in_bytes")]
    pub size_limit: usize,
}

#[derive(Deserialize, Clone)]
pub struct Storage {
    pub method: StorageMethod,
    pub directory: PathBuf,
    pub id_length: u16,
    pub expiration_in_hours: f32,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StorageMethod {
    File,
}

#[derive(Deserialize, Clone)]
pub struct Analyzer {
    pub custom_highlighting_delimiters: Vec<String>,
    pub lines_limits: AnalyzerLinesLimits,
}

#[derive(Deserialize, Clone)]
pub struct AnalyzerLinesLimits {
    pub server: usize,
    pub plugins: usize,
    pub ports: usize,
}

impl AnalyzerLinesLimits {
    pub fn max(&self) -> usize {
        self.server
            .max(self.server)
            .max(self.plugins)
            .max(self.ports)
    }
}

#[derive(Deserialize, Clone)]
pub struct Ports {
    pub plugins: Value,
    pub mods: Value,
}

#[derive(Deserialize, Clone)]
pub struct Api {
    pub public: bool,
    pub no_auth: bool,
}

#[derive(Deserialize, Clone)]
pub struct Redis {
    pub url: String,
    pub pool_size: usize,
}
