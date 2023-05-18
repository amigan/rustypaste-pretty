use crate::config::Config;
use actix_web::{web, HttpRequest, HttpResponse, Error, http::header};
use mime::Mime;
use text_template::*;
use std::collections::HashMap;
use std::str;

/// Render a pretty HttpResponse.
pub fn render_pretty(
    file: web::Path<String>,
    mime_type: Mime,
    config: &Config,
) -> Result<HttpResponse, Error> {
    let mut template_values = HashMap::new();
    let tmpl_bytes = str::from_utf8(include_bytes!("pretty.html")).unwrap();
    let tmpl = Template::from(tmpl_bytes);
    template_values.insert("file", file.as_str());
    template_values.insert("style", match &config.server.style {
        Some(style) => style.as_str(),
        None => "default",
    });
    let mime_str = mime_type.to_string();
    if let Some(overrides) = &config.paste.highlight_override {
        template_values.insert("type", if overrides.contains_key(&mime_str) { overrides[&mime_str].as_str() } else { "" });
    }
    let rendered = tmpl.fill_in(&template_values);

    Ok(HttpResponse::Ok().content_type(mime::TEXT_HTML).body(rendered.to_string()))
}

/// Check whether the request wants pretty mode.
pub fn want_pretty(
    request: &HttpRequest,
    by_default: bool,
) -> bool {
    if request.query_string() == "nopretty" {
        return false;
    }

    let mut accepts_html = false;

    if let Some(accept) = request.headers().get(header::ACCEPT) {
        accepts_html = accept.to_str().unwrap_or_default().find("text/html").is_some();
    }

    request.query_string() == "pretty" || (by_default && accepts_html)
}
