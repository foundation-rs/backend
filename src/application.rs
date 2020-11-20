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

// this function could be located in a different module
pub fn metainfo_config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(schemas_metainfo)
        .service(tables_metainfo);
}

#[derive(Serialize)]
struct MetainfoSchemas<'a> {
    schemas: Vec<&'a str>
}

#[get("/")]
async fn schemas_metainfo(data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let metainfo = data.metainfo.read().unwrap();
    let mut schemas:Vec<&str> = metainfo.schemas.keys().map(|k|k.as_ref() as &str).collect();
    schemas.sort();
    let response = MetainfoSchemas { schemas };
    HttpResponse::Ok().json(response)
}

#[derive(Serialize)]
struct MetainfoTables<'a> {
    tables: Vec<MetainfoTable<'a>>
}

#[derive(Serialize)]
struct MetainfoTable<'a> {
    name:      &'a str,
    is_view:   bool,
    temporary: bool,
    has_pk:    bool
}

#[get("/{schema}")]
async fn tables_metainfo(path: web::Path<(String,)>, data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let schema_name = path.into_inner().0.to_uppercase();
    let metainfo = data.metainfo.read().unwrap();
    let schema_info = metainfo.schemas.get(&schema_name);

    match schema_info {
        None => HttpResponse::NotFound().finish(),
        Some(info) => {
            let mut tables: Vec<MetainfoTable> = info.tables.values().map(|info|
                MetainfoTable {
                    name: info.name.as_ref(),
                    is_view: info.is_view,
                    temporary: info.temporary,
                    has_pk: info.primary_key.is_some()
                }).collect();
            tables.sort_by(|a,b|a.name.cmp(b.name));
            let response = MetainfoTables { tables };
            HttpResponse::Ok().json(response)
        }
    }
}

