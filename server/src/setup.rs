use slog_async;
use slog_term;

use slog::{Drain,o};
use openssl::ssl::{SslAcceptorBuilder, SslAcceptor, SslMethod, SslFiletype};
use std::path::Path;
use crate::config::HTTP;
use crate::application;
use std::fs::File;
use std::io::Read;

/// setup logging
// TODO: use logger everywhere
// TODO: async logger:  https://github.com/zupzup/rust-web-example/blob/main/src/logging/mod.rs
// TODO: actix example: https://www.zupzup.org/rust-webapp/index.html
// TODO: see also:      https://github.com/zupzup/rust-web-example/blob/main/src/handlers/mod.rs
// TODO: see also:      https://rust.graystorm.com/2019/07/20/better-logging-for-the-web-application/
pub fn logging() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(drain, o!())
}

/// load ssl keys
// to create a self-signed temporary cert for testing:
// `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
pub fn ssl(http: &HTTP) -> SslAcceptorBuilder {
    let ssl = &http.ssl;

    let keypath = Path::new(&ssl.path);
    let keyfilepath = keypath.join(&ssl.keyfile);
    let certfilepath = keypath.join(&ssl.certfile);

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    builder
        .set_private_key_file(keyfilepath, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(certfilepath).unwrap();
    builder
}

pub fn identity(http: &HTTP) -> crate::security::IdentityService {
    let ssl = &http.ssl;
    let jwt = &http.jwt;
    let token_name = jwt.cookie.to_string();

    let keypath = Path::new(&ssl.path);
    let keyfilepath = keypath.join(&jwt.publickey);

    crate::security::IdentityService::new(token_name, keyfilepath)
}

