use fred::types::RedisValue;
use serde::Serialize;
use tracing::log::warn;

pub struct PasteAnalyzer;

impl PasteAnalyzer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
    pub fn paste_type(&self, paste_content: &[u8]) -> PasteType {
        let length = paste_content.len();

        if length > 1_000_000 {
            // don't check for YAML/JSON when the paste is large
            return match self.is_log(paste_content) {
                true => PasteType::Log,
                false => PasteType::Other,
            };
        } else {
            // JSON needs to be first because any valid JSON is valid YAML
            if serde_json::from_slice::<serde_json::Value>(paste_content).is_ok() {
                return PasteType::Json;
            }

            if serde_yaml::from_slice::<serde_yaml::Value>(paste_content).is_ok() {
                return PasteType::Yaml;
            }

            if self.is_log(paste_content) {
                return PasteType::Log;
            }
        }

        PasteType::Other
    }

    fn is_log(&self, paste_content: &[u8]) -> bool {
        // TODO
        let s = String::from_utf8_lossy(paste_content);

        s.contains("INFO") || s.contains("WARN") || s.contains("ERROR") || s.contains("SEVERE")
    }
}

#[derive(Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PasteType {
    Log,
    Json,
    Yaml,
    Other,
}

impl From<PasteType> for RedisValue {
    fn from(paste_type: PasteType) -> Self {
        match paste_type {
            PasteType::Log => RedisValue::from_static_str("log"),
            PasteType::Json => RedisValue::from_static_str("json"),
            PasteType::Yaml => RedisValue::from_static_str("yaml"),
            PasteType::Other => RedisValue::from_static_str("other"),
        }
    }
}

impl From<String> for PasteType {
    fn from(paste_type: String) -> Self {
        match paste_type.as_str() {
            "log" => PasteType::Log,
            "json" => PasteType::Json,
            "yaml" => PasteType::Yaml,
            "other" => PasteType::Other,
            t => {
                warn!("Unknown PasteType value: '{t}'");
                PasteType::Other
            }
        }
    }
}
