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

pub struct PasteError;

impl IntoResponse for PasteError {
    fn into_response(self) -> Response {
        Redirect::to("/").into_response()
    }
}

#[derive(Template)]
#[template(path = "paste/log.html")]
struct LogTemplate<'a> {
    id: String,
    content: String,
    data: LogData,
    plugins: Vec<Plugin<'a>>,
}

#[derive(Template)]
#[template(path = "paste/yaml_json.html")]
struct YamlJsonTemplate {
    id: String,
    content: String,
    paste_type: PasteType,
}

#[derive(Template)]
#[template(path = "paste/invalid_yaml_json.html")]
struct InvalidYamlJsonTemplate {
    id: String,
    content: String,
    paste_type: PasteType,
    error_line: usize,
    error: Option<String>,
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

#[derive(Debug)]
struct ConfigParseError {
    message: String,
    line: Option<usize>,
    column: Option<usize>,
    context: Option<String>,
}

impl ConfigParseError {
    fn new(raw: &str, src: &str, line: Option<usize>, column: Option<usize>) -> Self {
        let message = raw
            .split_once(" at line ")
            .map_or(raw, |(msg, _)| msg)
            .to_owned();

        let context = match (line, column) {
            (Some(line), Some(col)) => Some(context(src, line, col, CONTEXT_RADIUS)),
            _ => None,
        };

        Self {
            message,
            line,
            column,
            context,
        }
    }

    fn format_parse_error(&self) -> String {
        let mut message = format!("Parser error: {}", self.message);

        if let (Some(line), Some(col)) = (self.line, self.column) {
            message.push_str(&format!("\nOn line {}, column {}", line, col));
        }

        if let Some(ctx) = &self.context {
            message.push_str(&format!("\n\n{}", ctx));
        }

        message
    }
}

fn context(src: &str, line: usize, col: usize, radius: usize) -> String {
    let start = line.saturating_sub(radius + 1);

    let mut out = String::new();

    for (idx, text) in src.lines().enumerate().skip(start).take(radius * 2 + 1) {
        let num = idx + 1;

        let marker = if num == line { ">" } else { " " };

        out.push_str(&format!("{marker} {num:>4} | {text}\n"));

        if num == line {
            out.push_str(&format!(
                "       | {}^\n",
                " ".repeat(col.saturating_sub(1))
            ));
        }
    }
    out
}

const CONTEXT_RADIUS: usize = 3;

fn parse_yaml(content: &str) -> Result<(), ConfigParseError> {
    serde_yaml::from_str::<serde_yaml::Value>(content)
        .map(|_| ())
        .map_err(|err| {
            let location = err.location();
            ConfigParseError::new(
                &err.to_string(),
                content,
                location.as_ref().map(|l| l.line()),
                location.map(|l| l.column()),
            )
        })
}

fn parse_json(content: &str) -> Result<(), ConfigParseError> {
    serde_json::from_str::<serde_json::Value>(content)
        .map(|_| ())
        .map_err(|err| {
            let (line, column) = if err.line() > 0 {
                (Some(err.line()), Some(err.column()))
            } else {
                (None, None)
            };
            ConfigParseError::new(&err.to_string(), content, line, column)
        })
}

async fn load_paste(
    storage_directory: &std::path::Path,
    id: &str,
) -> Result<(String, PasteMetadata), PasteError> {
    let paste_path = storage_directory.join(&id);

    let content = tokio::fs::read_to_string(&paste_path)
        .await
        .map_err(|err| {
            trace!(
                "Failed to read paste '{}' ({}) into String: {}",
                id,
                paste_path.display(),
                err
            );
            PasteError
        })?;

    let metadata_path = storage_directory.join("metadata").join(&id);

    let metadata_bytes: Vec<u8> = tokio::fs::read(&metadata_path).await.map_err(|err| {
        error!(
            "Failed to read paste '{}' metadata ({}): {}",
            id,
            metadata_path.display(),
            err
        );
        PasteError
    })?;

    let metadata = serde_json::from_slice::<PasteMetadata>(&metadata_bytes).map_err(|err| {
        error!(
            "Failed to parse metadata of '{}' into PasteMetadata: {}",
            id, err
        );
        PasteError
    })?;

    Ok((content, metadata))
}

fn render_log(id: String, lines: &[&str]) -> Response {
    /*
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
    */

    let static_analyzer = StaticAnalyzer::new(&lines);
    let plugins = static_analyzer.plugins(Some(1000)); // TODO sort plugins by name and filter out disabled ones
    let server_info = static_analyzer.version_platform(Some(750));

    let parser = Parser::new(&lines, Some(10000), Some(500), Some(15));

    let html = parser.html_lines();
    let content = html.concat();

    let template = LogTemplate {
        id,
        content,
        data: LogData {
            minecraft_version: server_info.minecraft_version,
            platform: server_info.platform,
            platform_version: server_info.platform_version,
            detections: vec![
                        /* Detection {
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
                    */],
        },
        plugins,
    };

    HtmlTemplate(template).into_response()
}

fn render_yaml_json(id: String, lines: &[&str], metadata: PasteMetadata) -> Response {
    let parser = NoLogParser::new(&lines);

    let html = parser.html();
    let content = html.concat();

    let template = YamlJsonTemplate {
        id,
        content,
        paste_type: metadata.paste_type,
    };

    HtmlTemplate(template).into_response()
}

fn render_invalid_yaml_json(
    id: String,
    lines: &[&str],
    content: &str,
    metadata: PasteMetadata,
) -> Response {
    let error = match metadata.paste_type {
        PasteType::InvalidYaml => parse_yaml(content).err(),
        PasteType::InvalidJson => parse_json(content).err(),
        _ => unreachable!(),
    };

    let parser = NoLogParser::new(&lines);

    let html = parser.html();
    let content = html.concat();

    let error_line = error.as_ref().and_then(|err| err.line).unwrap_or(0);

    let template = InvalidYamlJsonTemplate {
        id,
        content,
        paste_type: metadata.paste_type,
        error_line,
        error: error.map(|err| err.format_parse_error()),
    };

    HtmlTemplate(template).into_response()
}

fn render_other(id: String, lines: &[&str]) -> Response {
    let parser = NoLogParser::new(&lines);

    let html = parser.html();
    let content = html.concat();

    let template = OtherTemplate { id, content };

    HtmlTemplate(template).into_response()
}

#[tracing::instrument(skip(state))]
pub async fn paste_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, PasteError> {
    let storage_directory = state.configuration.storage.directory;

    let (content, metadata) = load_paste(&storage_directory, &id).await?;

    let lines: Vec<&str> = content.lines().collect();

    Ok(match metadata.paste_type {
        PasteType::Log => render_log(id, &lines),
        PasteType::Yaml | PasteType::Json => render_yaml_json(id, &lines, metadata),
        PasteType::InvalidYaml | PasteType::InvalidJson => {
            render_invalid_yaml_json(id, &lines, &content, metadata)
        }
        PasteType::Other => render_other(id, &lines),
    })
}

struct NoLogParser<'a> {
    lines: &'a [&'a str],
}

impl<'a> NoLogParser<'a> {
    fn new(lines: &'a [&'a str]) -> Self {
        Self { lines }
    }

    fn html(&self) -> Vec<String> {
        let mut html = Vec::new();

        for (idx, line) in self.lines.iter().enumerate() {
            let line = format!(
                r#"<span class="p" id="L{}">{}</span>"#,
                idx + 1,
                html_escape::encode_text(line)
            );

            html.push(line);
        }

        html
    }
}
