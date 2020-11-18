#![feature(min_const_generics)]
#![feature(option_insert)]

use std::env;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::io::{Error, ErrorKind};

mod config;
mod metainfo;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let start = chrono::offset::Local::now();

    let ref conf = config::load("config.xml")
        .map_err(|e|Error::new(ErrorKind::Other, e))?;

    let mi = metainfo::MetaInfo::load(&conf)
        .map_err(|e|Error::new(ErrorKind::Other, e))?;

    let server = HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind("127.0.0.1:8081")?
        .run();

    println!();
    println!("SERVER STARTED ON `localhost:8081`");
    let end = chrono::offset::Local::now();
    let duration = end - start;

    let seconds = duration.num_seconds();
    let milliseconds = duration.num_milliseconds() - seconds * 1000;
    println!();
    println!("ELAPSED: {} seconds, {} milliseconds", seconds, milliseconds);

    // info!(log, "Server Started on localhost:8081");
    server.await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
