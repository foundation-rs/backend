use std::sync::{Arc, RwLock};
use std::io::{Error, ErrorKind, Result};

use serde::{Serialize};

use crate::config::Config;
use crate::metainfo::{self,MetaInfo};
use actix_web::{get, web, Responder, HttpResponse};

// This struct represents state
pub struct ApplicationState {
    metainfo: RwLock<MetaInfo>
}

impl ApplicationState {
    pub fn load(conf: &Config) -> Result<ApplicationState> {
        let metainfo = metainfo::MetaInfo::load(&conf.excludes)
            .map_err(|e|Error::new(ErrorKind::Other, e))?;
        let metainfo = RwLock::new(metainfo);
        Ok( ApplicationState{metainfo} )
    }
}

#[derive(Serialize)]
struct MetainfoSchemas<'a> {
    schemas: Vec<&'a str>
}

#[get("/metainfo")]
pub async fn get_metainfo(data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let metainfo = data.metainfo.read().unwrap();
    let mut schemas:Vec<&str> = metainfo.schemas.keys().map(|k|k.as_ref() as &str).collect();
    schemas.sort();
    let response = MetainfoSchemas { schemas };
    HttpResponse::Ok().json(response)
}


