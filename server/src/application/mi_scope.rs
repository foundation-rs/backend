use std::sync::Arc;
use actix_web::{Scope, web, Responder, HttpResponse};
use crate::application::ApplicationState;

// group of endpoints for metainfo
pub fn metainfo_scope() -> Scope {
    web::scope("/metainfo")
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
