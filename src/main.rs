#![feature(min_const_generics)]
#![feature(option_insert)]

use std::io::{Error, ErrorKind};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_slog::StructuredLogger;
use slog::info;

mod config;
mod datasource;
mod setup;
mod metainfo;
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

    let mi = metainfo::MetaInfo::load(&conf.excludes)
        .map_err(|e|Error::new(ErrorKind::Other, e))?;

    info!(log, "Server Started on {}", &http.listen);

    HttpServer::new(move || {
        App::new()
            .wrap(StructuredLogger::new(log.clone()))

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

