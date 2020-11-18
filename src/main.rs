#![feature(min_const_generics)]
#![feature(option_insert)]

use std::env;
use std::io::{Error, ErrorKind};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use env_logger::Env;

mod config;
mod datasource;
mod metainfo;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // TODO: use logger everywhere
    // TODO: async logger: https://github.com/zupzup/rust-web-example/blob/main/src/logging/mod.rs

    // TODO: http2 and ssl

    let start = chrono::offset::Local::now();

    let ref conf = config::load("config.xml")
        .map_err(|e|Error::new(ErrorKind::Other, e))?;

    datasource::create(&conf.connection)
        .map_err(|e|Error::new(ErrorKind::Other, e))?;

    let mi = metainfo::MetaInfo::load(&conf.excludes)
        .map_err(|e|Error::new(ErrorKind::Other, e))?;

    let end = chrono::offset::Local::now();
    let duration = end - start;

    let seconds = duration.num_seconds();
    let milliseconds = duration.num_milliseconds() - seconds * 1000;
    println!();
    println!("ELAPSED: {} seconds, {} milliseconds", seconds, milliseconds);
    println!();

    let listen: &str = &conf.http.listen;

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))

            .service(hello)
            .service(echo)
            .service(health)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind(listen)?
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

