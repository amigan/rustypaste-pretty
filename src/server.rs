use crate::config::Config;
use crate::file;
use crate::header::ContentDisposition;
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::http::header::AUTHORIZATION;
use actix_web::{error, get, post, web, Error, HttpRequest, HttpResponse, Responder};
use byte_unit::Byte;
use futures_util::stream::StreamExt;
use std::convert::TryFrom;

/// Shows the landing page.
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("oops!")
}

/// Serves a file from the upload directory.
#[get("/{file}")]
async fn serve(
    request: HttpRequest,
    path: web::Path<String>,
    config: web::Data<Config>,
) -> Result<HttpResponse, Error> {
    let path = config.server.upload_path.join(&*path);
    let file = NamedFile::open(&path)?
        .disable_content_disposition()
        .prefer_utf8(true);
    let response = file.into_response(&request)?;
    Ok(response)
}

/// Handles file upload by processing `multipart/form-data`.
#[post("/")]
async fn upload(
    request: HttpRequest,
    mut payload: Multipart,
    config: web::Data<Config>,
) -> Result<HttpResponse, Error> {
    if let Some(token) = &config.server.auth_token {
        if request
            .headers()
            .get(AUTHORIZATION)
            .map(|v| v.to_str().unwrap_or_default())
            .map(|v| v.split_whitespace().last().unwrap_or_default())
            != Some(token)
        {
            return Err(error::ErrorUnauthorized("unauthorized"));
        }
    }
    let mut urls: Vec<String> = Vec::new();
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content = ContentDisposition::try_from(field.content_disposition())?;
        if content.has_form_field("file") {
            let mut bytes = Vec::<u8>::new();
            while let Some(chunk) = field.next().await {
                bytes.append(&mut chunk?.to_vec());
            }
            if bytes.len() as u128 > config.server.max_content_length.get_bytes() {
                return Err(error::ErrorPayloadTooLarge("upload limit exceeded"));
            }
            let file_name = &file::save(content.get_file_name()?, &bytes, &config)?;
            let connection = request.connection_info();
            log::info!(
                "{} ({}) is uploaded from {}",
                file_name,
                Byte::from_bytes(bytes.len() as u128).get_appropriate_unit(false),
                connection.remote_addr().unwrap_or("unknown host")
            );
            urls.push(format!(
                "{}://{}/{}\n",
                connection.scheme(),
                connection.host(),
                file_name
            ));
        } else {
            return Err(error::ErrorUnprocessableEntity("invalid form parameters"));
        }
    }
    Ok(HttpResponse::Ok().body(urls.join("")))
}

/// Configures the server routes.
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index)
        .service(serve)
        .service(upload)
        .route("", web::head().to(HttpResponse::MethodNotAllowed));
}
