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
use tracing::trace;

#[derive(Template)]
#[template(path = "paste/log.html")]
struct PasteTemplate<'a> {
    id: String,
    data: LogData,
    plugins: Vec<Plugin<'a>>,
}

pub struct LogData {
    pub minecraft_version: Option<String>,
    pub platform: Platform,
    pub platform_version: Option<String>,
    pub detections: Vec<Detection>,
    pub content: String,
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
        "Vypněte server".to_string(),
        "Otevřete Správce Souborů/FTP".to_string(),
        "Smažte složku world".to_string(),
    ];

    let solutions = Solutions {
        format,
        lines: lines.clone(),
    };

    let solutionss = Solutions {
        format: SolutionFormat::default(),
        lines: vec!["V `server.properties` nastavte `level-name`".to_string()],
    };

    let solutionsss = solutionss.clone();

    let paste_path = state.configuration.storage.directory.join(&id);

    let content = match tokio::fs::read_to_string(&paste_path).await {
        Ok(content) => content,
        Err(err) => {
            trace!(
                "Failed to read paste '{}' into String: {}",
                paste_path.display(),
                err
            );
            return Redirect::to("/").into_response();
        }
    };

    let lines: Vec<&str> = content.lines().collect();

    let liness = content.lines().map(String::from).collect();
    let static_analyzer = StaticAnalyzer::new(liness);
    let plugins = static_analyzer.plugins(None);
    let server_info = static_analyzer.version_platform(None);
    println!("{:?}", server_info);
    // TODO sort plugins by name and filter out disabled ones

    let parser = Parser::new(&lines, Some(10000), Some(500), Some(15));
    let html = parser.html_lines();

    let bytes: Vec<u8> = html.iter().flat_map(|entry| entry.bytes()).collect();

    let template = PasteTemplate {
        id,
        data: LogData {
            minecraft_version: server_info.minecraft_version,
            platform: server_info.platform,
            platform_version: server_info.platform_version,
            detections: vec![
                Detection {
                    header: "Zjištěn zastaralý plugin".to_string(),
                    detail: None,
                    solutions: Some(solutions),
                    line: 1,
                },
                Detection {
                    header: "Svět nemá jméno".to_string(),
                    detail: None,
                    solutions: Some(solutionss),
                    line: 3,
                },
                Detection {
                    header: "Svět má jméno".to_string(),
                    detail: Some("https://x.com/caukub".to_string()),
                    solutions: Some(solutionsss),
                    line: 3,
                },
            ],
            content: String::from_utf8(bytes).unwrap(),
        },
        plugins,
    };

    HtmlTemplate(template).into_response()
}
