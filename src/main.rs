#![feature(min_const_generics)]
#![feature(option_insert)]

use std::io::{Error, ErrorKind};
use std::path::Path;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use env_logger::Env;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod config;
mod datasource;
mod metainfo;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // TODO: use logger everywhere
    // TODO: async logger:  https://github.com/zupzup/rust-web-example/blob/main/src/logging/mod.rs
    // TODO: actix example: https://www.zupzup.org/rust-webapp/index.html
    // TODO: see also:      https://github.com/zupzup/rust-web-example/blob/main/src/handlers/mod.rs

    let start = chrono::offset::Local::now();

    let ref conf = config::load("config.xml")
        .map_err(|e|Error::new(ErrorKind::Other, e))?;

    let http = &conf.http;
    // load ssl keys
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let ssl = &http.ssl;

    let keypath = Path::new(&ssl.path);
    let keyfilepath = keypath.join(&ssl.keyfile);
    let certfilepath = keypath.join(&ssl.certfile);

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    builder
        .set_private_key_file(keyfilepath, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(certfilepath).unwrap();

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

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))

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

