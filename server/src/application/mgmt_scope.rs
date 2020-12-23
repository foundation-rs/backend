use std::sync::Arc;
use actix_web::{get, web, Scope, Responder, HttpResponse};
use serde::Serialize;

use crate::application::ApplicationState;
use crate::application;
use actix_web::dev::HttpServiceFactory;
use std::collections::HashSet;
use std::iter::FromIterator;

// group of endpoints for metainfo
pub fn management_scope() -> impl HttpServiceFactory {
    web::scope("/mgmt")
        .wrap(crate::security::Authorized::developers())
        .service(schemas_metainfo)
        .service(tables_metainfo)
        .service(table_metainfo)
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
    pub col_type: &'static str,
    pub is_pk:    bool,
    pub nullable: bool
}

#[get("/schemas")]
async fn schemas_metainfo(data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let metainfo = data.metainfo.read().unwrap();
    let mut schemas: Vec<&str> = metainfo.schemas.iter().map(|s|s.name.as_str()).collect();
    schemas.sort();
    let response = DatabaseMetainfo { schemas };
    HttpResponse::Ok().json(response)
}

#[get("/schemas/{schema}")]
async fn tables_metainfo(path: web::Path<(String,)>, data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let schema_name = path.into_inner().0;
    let metainfo = data.metainfo.read().unwrap();

    match metainfo.schemas.get(schema_name.as_str()) {
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

#[get("/schemas/{schema}/{table}")]
async fn table_metainfo(path: web::Path<(String,String)>, data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let (schema_name,table_name) = path.into_inner();
    let metainfo = data.metainfo.read().unwrap();

    if let Some(info) = metainfo.schemas.get(schema_name.as_str()) {
        if let Some(info) = info.tables.get(table_name.as_str()) {
            let pk_indices = match &info.primary_key {
              Some(pk) => {
                  HashSet::from_iter(&pk.column_indices)
              }, None => {
                    HashSet::new()
                }
            };

            let columns = info
                .columns
                .iter()
                .enumerate()
                .map(|(ref i, c)| {
                    let is_pk = pk_indices.contains(i);
                    ColumnMetaInfo { name: c.name.as_str(), col_type: c.col_type_name, is_pk, nullable: c.nullable}
                }).collect();

            let response = TableMetaInfo {
                name: info.name.as_str(),
                is_view: info.is_view,
                temporary: info.temporary,
                has_pk: pk_indices.len() > 0,
                columns
            };
            return HttpResponse::Ok().json(response)
        }
    };

    HttpResponse::NotFound().finish()
}
