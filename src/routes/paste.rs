use askama::Template;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
};
use mclog::{
    analyzer::static_analyzer::{
        StaticAnalyzer,
        plugin::{Plugin, StaticPluginAnalyzer},
        server::{Platform, StaticServerAnalyzer},
    },
    parser::Parser,
};

use crate::{AppState, HtmlTemplate};
use tracing::{error, trace};

use super::{PasteMetadata, PasteType};

#[derive(Template)]
#[template(path = "paste/log.html")]
struct LogTemplate<'a> {
    id: String,
    content: String,
    data: LogData,
    plugins: Vec<Plugin<'a>>,
}

#[derive(PartialEq)]
enum ConfigFormat {
    Yaml,
    Json,
}

#[derive(Template)]
#[template(path = "paste/yaml_json.html")]
struct YamlJsonTemplate {
    id: String,
    content: String,
    format: ConfigFormat,
}

#[derive(Template)]
#[template(path = "paste/invalid_yaml_json.html")]
struct InvalidYamlJsonTemplate {
    id: String,
    content: String,
    format: ConfigFormat,
}

#[derive(Template)]
#[template(path = "paste/other.html")]
struct OtherTemplate {
    id: String,
    content: String,
}

pub struct LogData {
    pub minecraft_version: Option<String>,
    pub platform: Platform,
    pub platform_version: Option<String>,
    pub detections: Vec<Detection>,
}

pub struct Detection {
    pub header: String,
    pub detail: Option<String>,
    pub solutions: Option<Solutions>,
    pub line: usize,
}

#[derive(Clone)]
pub struct Solutions {
    format: SolutionFormat,
    lines: Vec<String>,
}

#[derive(Default, Clone)]
pub enum SolutionFormat {
    Or,
    OrderedList,
    #[default]
    Raw,
}

#[tracing::instrument(skip(state))]
pub async fn paste_handler(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    let format = SolutionFormat::OrderedList;
    let lines = vec![
        "Turn off the server".to_string(),
        "Open the File Manager/FTP".to_string(),
        "Delete the world folder (backup before if needed)".to_string(),
    ];

    let solutions = Solutions {
        format,
        lines: lines.clone(),
    };

    let solutionss = Solutions {
        format: SolutionFormat::default(),
        lines: vec!["In the `server.properties` file, set the value of `level-name`.".to_string()],
    };

    let storage_directory = state.configuration.storage.directory;

    let paste_path = storage_directory.join(&id);

    let content = match tokio::fs::read_to_string(&paste_path).await {
        Ok(content) => content,
        Err(err) => {
            trace!(
                "Failed to read paste '{}' ({}) into String: {}",
                id,
                paste_path.display(),
                err
            );
            return Redirect::to("/").into_response();
        }
    };

    let paste_metadata = storage_directory.join("metadata").join(&id);

    let metadata_bytes = match tokio::fs::read(&paste_metadata).await {
        Ok(bytes) => bytes,
        Err(err) => {
            error!(
                "Failed to read paste '{}' metadata ({}): {}",
                id,
                paste_metadata.display(),
                err
            );
            return Redirect::to("/").into_response();
        }
    };

    let metadata = match serde_json::from_slice::<PasteMetadata>(&metadata_bytes) {
        Ok(metadata) => metadata,
        Err(err) => {
            error!(
                "Failed to parse metadata of '{}' into PasteMetadata: {}",
                id, err
            );
            return Redirect::to("/").into_response();
        }
    };

    let lines: Vec<&str> = content.lines().collect();

    match metadata.paste_type {
        PasteType::Log => {
            let static_analyzer = StaticAnalyzer::new(lines.clone());
            let plugins = static_analyzer.plugins(Some(1000)); // TODO sort plugins by name and filter out disabled ones
            let server_info = static_analyzer.version_platform(Some(750));

            let parser = Parser::new(&lines, Some(10000), Some(500), Some(15));
            let html = parser.html_lines();

            let bytes: Vec<u8> = html.iter().flat_map(|entry| entry.bytes()).collect();

            let content = String::from_utf8(bytes).unwrap();

            let template = LogTemplate {
                id,
                content,
                data: LogData {
                    minecraft_version: server_info.minecraft_version,
                    platform: server_info.platform,
                    platform_version: server_info.platform_version,
                    detections: vec![
                        Detection {
                            header: "Loading a world from a newer version in an older version"
                                .to_string(),
                            detail: None,
                            solutions: Some(solutions),
                            line: 1,
                        },
                        Detection {
                            header: "World does not have a name".to_string(),
                            detail: None,
                            solutions: Some(solutionss),
                            line: 3,
                        },
                    ],
                },
                plugins,
            };

            HtmlTemplate(template).into_response()
        }
        PasteType::Yaml | PasteType::Json => {
            let parser = NoLogParser::new(lines);
            let html = parser.html();
            
            let bytes: Vec<u8> = html.iter().flat_map(|entry| entry.bytes()).collect();
            let content = String::from_utf8(bytes).unwrap();

            let format = match metadata.paste_type {
                PasteType::Yaml => ConfigFormat::Yaml,
                PasteType::Json => ConfigFormat::Json,
                _ => unreachable!(),
            };

            let template = YamlJsonTemplate {
                id,
                content,
                format,
            };

            HtmlTemplate(template).into_response()
        }
        PasteType::InvalidYaml | PasteType::InvalidJson => {
            let parser = NoLogParser::new(lines);
            let html = parser.html();

            let bytes: Vec<u8> = html.iter().flat_map(|entry| entry.bytes()).collect();
            let content = String::from_utf8(bytes).unwrap();

            let format = match metadata.paste_type {
                PasteType::InvalidYaml => ConfigFormat::Yaml,
                PasteType::InvalidJson => ConfigFormat::Json,
                _ => unreachable!(),
            };

            let template = InvalidYamlJsonTemplate {
                id,
                content,
                format
            };

            HtmlTemplate(template).into_response()
        }
        PasteType::Other => {
            let parser = NoLogParser::new(lines);
            let html = parser.html();

            let bytes: Vec<u8> = html.iter().flat_map(|entry| entry.bytes()).collect();
            let content = String::from_utf8(bytes).unwrap();

            let template = OtherTemplate { id, content };

            HtmlTemplate(template).into_response()
        }
    }
}

struct NoLogParser<'a> {
    lines: Vec<&'a str>,
}

impl<'a> NoLogParser<'a> {
    fn new(lines: Vec<&'a str>) -> Self {
        Self { lines }
    }

    fn html(&self) -> Vec<String> {
        let mut html = Vec::new();

        for (idx, line) in self.lines.iter().enumerate() {
            let line = format!(r#"<span class="p" id="L{}">{}</span>"#, idx + 1, line);

            html.push(line);
        }

        html
    }
}
