use crate::paste::analyzer::PasteType;
use crate::paste::Paste;
use crate::routes::api::{get_analyzer_details, get_paste_lines, ApiError};
use crate::routes::get::get_paste_type;
use crate::AppState;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use mclog::analyzer::dynamic::chunks::Captures;
use mclog::analyzer::dynamic::{Detection, DynamicAnalyzer, ScriptPlatform};
use mclog::analyzer::{DynamicAnalyzerDetails, Platform};
use mclog::parser::parser::Parser;
use rhai::{Dynamic, Scope};
use serde::Serialize;
use tracing::{error, warn};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoLogResponse {
    content: String,
    paste_type: PasteType,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogResponse {
    content: String,
    paste_type: PasteType,
    version: String,
    platform: Platform,
    detections: Vec<Detection>,
}

pub async fn get_paste(
    State(app_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let paste = Paste::new(id.clone(), app_state.configuration.clone());
    let paste_type = get_paste_type(id.clone(), &paste, app_state.clone())
        .await
        .map_err(|_| ApiError::Server("Couldn't get PasteType"))?;

    let limits = app_state.configuration.analyzer.lines_limits.clone();

    if let PasteType::Log = paste_type {
        let lines = get_paste_lines(id.clone(), &app_state.configuration, limits.max()).await?;
        let parser = Parser::new(
            lines,
            app_state
                .configuration
                .analyzer
                .custom_highlighting_delimiters
                .clone(),
        );

        let html_as_bytes = parsed_html_log(parser);

        let details = get_analyzer_details(
            id.clone(),
            &app_state.configuration,
            limits.plugins,
            limits.ports,
        )
        .await?;

        let html = String::from_utf8_lossy(&html_as_bytes).to_string();

        let response = LogResponse {
            version: details
                .version
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            platform: details.platform,
            detections: get_detections(details.clone()),
            content: html,
            paste_type,
        };

        Ok(Json(response).into_response())
    } else {
        let content =
            tokio::fs::read_to_string(app_state.configuration.storage.directory.join(id.clone()))
                .await
                .map_err(|_| ApiError::Client("Couldn't read file to String"))?;

        let response = NoLogResponse {
            content: html_escape::encode_text(&content).to_string(),
            paste_type,
        };

        Ok(Json(response).into_response())
    }
}

fn parsed_html_log(parser: Parser) -> Vec<u8> {
    let html = parser.html();

    html.iter().flat_map(|entry| entry.bytes()).collect()
}

fn get_detections(dad: DynamicAnalyzerDetails) -> Vec<Detection> {
    let engine = DynamicAnalyzer::default();
    let mut scripts = Vec::new();
    scripts.append(&mut engine.scripts(ScriptPlatform::Global));

    match dad.platform {
        Platform::Vanilla => {}
        Platform::CraftBukkit
        | Platform::Spigot
        | Platform::Paper
        | Platform::Pufferfish
        | Platform::Purpur => scripts.append(&mut engine.scripts(ScriptPlatform::Bukkit)),
        Platform::Fabric => scripts.append(&mut engine.scripts(ScriptPlatform::Fabric)),
        Platform::Forge => scripts.append(&mut engine.scripts(ScriptPlatform::Forge)),
        Platform::BungeeCord | Platform::Waterfall => {
            scripts.append(&mut engine.scripts(ScriptPlatform::BungeeCord))
        }
        Platform::Velocity => scripts.append(&mut engine.scripts(ScriptPlatform::Velocity)),
    }

    if !dad.is_proxy {
        scripts.append(&mut engine.scripts(ScriptPlatform::NoProxy));
    }

    let mut scope = Scope::new();
    scope.push_constant("dad", dad.clone());

    let mut detections = Vec::new();

    for script in scripts {
        let result = match engine
            .engine
            .eval_ast_with_scope::<Dynamic>(&mut scope, &script.ast)
        {
            Ok(return_code) => return_code,
            Err(err) => {
                error!("{} in file {}", err, script.file);
                Dynamic::UNIT
            }
        };

        if result.is_unit() {
            continue;
        };

        if result.is_string() {
            let result = result.to_string();

            match script.detection.get(&result) {
                None => {
                    error!(
                        "Error while obtaining detection info for {} from file {}",
                        result, script.file
                    );
                    continue;
                }
                Some(det) => {
                    let det = det.to_owned();

                    if det.private.is_some_and(|is_private| is_private) {
                        continue;
                    }
                    detections.push(det.clone())
                }
            }
        } else if let Some(guard) = result.read_lock::<Vec<Captures>>() {
            let results: Vec<Captures> = guard.to_owned();

            for result in results {
                match script.detection.get(&result.identifier) {
                    None => {
                        error!(
                            "Error while obtaining detection info for {:?} from file {}",
                            result, script.file
                        );
                        continue;
                    }
                    Some(det) => {
                        let mut det = det.to_owned();

                        for capture in result.captures.iter() {
                            det.header = det.header.replace(
                                &format!("{{{}}}", capture.0),
                                result
                                    .captures
                                    .get(capture.0)
                                    .expect("Capture doesn't exist, this should not happen"),
                            );
                        }

                        for (idx, _) in det.clone().solutions.into_iter().enumerate() {
                            for capture in result.captures.iter() {
                                det.solutions[idx] = det.clone().solutions[idx].clone().replace(
                                    &format!("{{{}}}", capture.0),
                                    result
                                        .captures
                                        .get(capture.0)
                                        .expect("Capture doesn't exist, this should not happen"),
                                );
                            }
                        }

                        if det.private.is_some_and(|is_private| is_private) {
                            continue;
                        }
                        detections.push(det.clone())
                    }
                }
            }
        } else {
            warn!("Unexpected result type for {:?}, skipping..", result);
            continue;
        }
    }
    detections.sort_by_key(|item| item.level);

    detections
}
