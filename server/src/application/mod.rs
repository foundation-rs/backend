mod mi_scope;
mod api_scope;

use std::sync::{Arc, RwLock};
use std::io::{Error, ErrorKind, Result};

// TODO: full static files support with NPM build
// SEE:  https://crates.io/crates/actix-web-static-files

// TODO: analize example https://github.com/actix/examples/blob/master/basics/src/main.rs
// TODO: example with static files and R2D2: https://stackoverflow.com/questions/63653540/serving-static-files-with-actix-web-2-0

use actix_web::{get, web, HttpResponse, Responder, Scope, HttpRequest};
use actix_files as fs;
use serde::Serialize;

use crate::config::Config;
use crate::metainfo::{self, MetaInfo};
use actix_files::NamedFile;
use std::path::PathBuf;

pub use mi_scope::metainfo_scope;
pub use api_scope::api_scope;

// This struct represents state
pub struct ApplicationState {
    metainfo: RwLock<MetaInfo>
}

impl ApplicationState {
    pub fn load(conf: &Config) -> Result<Arc<ApplicationState>> {
        let metainfo = metainfo::MetaInfo::load(&conf.excludes)
            .map_err(|e|Error::new(ErrorKind::Other, e))?;
        let metainfo = RwLock::new(metainfo);
        Ok( Arc::new(ApplicationState{metainfo}) )
    }
}

// group of base endpoints
pub fn base_scope() -> Scope {
    web::scope("/")
        .service(health)
        .service(fs::Files::new("/", "./www")
            .show_files_listing()
            .use_last_modified(true))
        .default_service(web::resource("").route(web::get().to(index)))
}

async fn index() -> Result<NamedFile> {
    let path: PathBuf = "./www/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[get("/health")]
async fn health() -> impl Responder {
    "OK".to_string()
}
