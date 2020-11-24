use std::sync::{Arc, RwLock};
use std::io::{Error, ErrorKind, Result};

use actix_web::{get, web, HttpResponse, Responder, Scope};
use serde::{Serialize};

use crate::config::Config;
use crate::metainfo::{self, MetaInfo};

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

// group of base endpoints
pub fn base_scope() -> Scope {
    web::scope("/")
        .service(hello)
        .service(health)
}

#[get("")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/health")]
async fn health() -> impl Responder {
    "OK".to_string()
}

// group of endpoints for metainfo
pub fn metainfo_scope() -> Scope {
    web::scope("/metainfo")
        .service(schemas_metainfo)
        .service(tables_metainfo)
        .service(table_metainfo)
}

// group of endpoints for api
pub fn api_scope() -> Scope {
    web::scope("/api")
        .service(table_query_by_pk)
}

#[derive(Serialize)]
struct DatabaseMetainfo<'a> {
    schemas: Vec<&'a str>
}

#[derive(Serialize)]
struct SchemaMetainfo<'a> {
    tables: Vec<TableMetaInfoBrief<'a>>
}

#[derive(Serialize)]
struct TableMetaInfoBrief<'a> {
    name:      &'a str,
    is_view:   bool,
    temporary: bool,
    has_pk:    bool
}

#[derive(Serialize)]
struct TableMetaInfo<'a> {
    name:      &'a str,
    is_view:   bool,
    temporary: bool,
    has_pk:    bool,
    columns:   Vec<ColumnMetaInfo<'a>>
}

#[derive(Serialize)]
pub struct ColumnMetaInfo<'a> {
    pub name:     &'a str,
    pub col_type: &'a str,
    pub nullable: bool
}

#[get("/")]
async fn schemas_metainfo(data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let metainfo = data.metainfo.read().unwrap();
    let mut schemas: Vec<&str> = metainfo.schemas.iter().map(|s|s.name.as_str()).collect();
    schemas.sort();
    let response = DatabaseMetainfo { schemas };
    HttpResponse::Ok().json(response)
}

#[get("/{schema}")]
async fn tables_metainfo(path: web::Path<(String,)>, data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let schema_name = path.into_inner().0;
    let metainfo = data.metainfo.read().unwrap();

    match metainfo.schemas.get(schema_name.to_uppercase().as_str()) {
        Some(info) => {
            let mut tables: Vec<TableMetaInfoBrief> = info.tables.iter().map(|info|
                TableMetaInfoBrief {
                    name: info.name.as_str(),
                    is_view: info.is_view,
                    temporary: info.temporary,
                    has_pk: info.primary_key.is_some()
                }).collect();
            tables.sort_by(|a,b|a.name.cmp(b.name));

            HttpResponse::Ok().json(SchemaMetainfo { tables })
        },
        None => HttpResponse::NotFound().finish()
    }
}

#[get("/{schema}/{table}")]
async fn table_metainfo(path: web::Path<(String,String)>, data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let (schema_name,table_name) = path.into_inner();
    let metainfo = data.metainfo.read().unwrap();

    if let Some(info) = metainfo.schemas.get(schema_name.to_uppercase().as_str()) {
        if let Some(info) = info.tables.get(table_name.to_uppercase().as_str()) {
            let columns = info
                .columns
                .iter()
                .map(|c| ColumnMetaInfo { name: c.name.as_str(), col_type: c.col_type_name.as_str(), nullable: c.nullable}).collect();

            let response = TableMetaInfo {
                name: info.name.as_str(),
                is_view: info.is_view,
                temporary: info.temporary,
                has_pk: info.primary_key.is_some(),
                columns
            };
            return HttpResponse::Ok().json(response)
        }
    };

    HttpResponse::NotFound().finish()
}

#[get("/{schema}/{table}/{pk}")]
async fn table_query_by_pk(path: web::Path<(String,String,String)>, data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let (schema_name,table_name, primary_key) = path.into_inner();
    let metainfo = data.metainfo.read().unwrap();

    if let Some(info) = metainfo.schemas.get(schema_name.to_uppercase().as_str()) {
        if let Some(info) = info.tables.get(table_name.to_uppercase().as_str()) {
            return match &info.primary_key {
                Some(pk) if pk.columns.len() == 1 => {
                    let select = generate_select_by_pk(
                        &info.name,
                        pk.columns.get(0).unwrap(),
                        info.columns.iter().map(|c|c.name.as_str()).collect());
                    HttpResponse::Ok().body(select)
                },
                _ => HttpResponse::BadRequest().finish()
            };
        }
    };

    HttpResponse::NotFound().finish()
}

fn generate_select_by_pk(table_name: &str, pk_column_name: &str, columns: Vec<&str>) -> String {
    format!("SELECT {} FROM {} WHERE {} = :1", columns.join(","), table_name, pk_column_name)
}
