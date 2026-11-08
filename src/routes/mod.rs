pub mod index;
pub mod paste;
pub mod raw;
pub mod upload;

pub use index::*;
pub use paste::*;
pub use raw::*;
use serde::{Deserialize, Serialize};
pub use upload::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PasteType {
    Log,
    Yaml,
    Json,
    InvalidYaml,
    InvalidJson,
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PasteMetadata {
    #[serde(rename = "type")]
    pub paste_type: PasteType,
}
