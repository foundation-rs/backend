use std::sync::Arc;
use std::io::{Error, ErrorKind};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, middleware};
use actix_web::http::ContentEncoding;
use actix_slog::StructuredLogger;
use slog::info;

mod application;
mod config;
mod datasource;
mod metainfo;
mod setup;
mod utils;

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

            .service(web::scope("/metainfo").configure(application::metainfo_config))
            .service(hello)
            .service(echo)
            .service(health)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind_openssl(&http.listen, builder)?
        .run()
        .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/health")]
async fn health() -> impl Responder {
    "OK".to_string()
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

