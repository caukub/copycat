use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::HtmlTemplate;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

pub async fn index_handler() -> Response {
    let template = IndexTemplate;

    HtmlTemplate(template).into_response()
}
