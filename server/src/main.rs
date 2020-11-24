use std::sync::Arc;
use std::io::{Error, ErrorKind};

use actix_web::{middleware, App, HttpServer};
use actix_web::http::ContentEncoding;
use actix_slog::StructuredLogger;
use slog::info;

mod application;
mod config;
mod datasource;
mod metainfo;
mod setup;
mod utils;

// TODO: threadlocal: https://doc.rust-lang.org/std/macro.thread_local.html

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log = setup::logging();
    info!(log, "Starting Foundation Server");

    let ref conf = config::load("config.xml")
        .map_err(|e|Error::new(ErrorKind::Other, e))?;

    let http = &conf.http;
    let builder = setup::ssl(&http);

    datasource::create(&conf.connection)
        .map_err(|e|Error::new(ErrorKind::Other, e))?;

    let application = Arc::new(application::ApplicationState::load(&conf)? );

    info!(log, "Server Started on https://{}", &http.listen);

    HttpServer::new(move || {
        App::new()
            .data(application.clone())
            .wrap(StructuredLogger::new(log.clone()))
            .wrap(middleware::Compress::new(ContentEncoding::Br))

            .service(application::metainfo_scope())
            .service(application::api_scope())
            .service( application::base_scope())
    })
        .bind_openssl(&http.listen, builder)?
        .run()
        .await
}
