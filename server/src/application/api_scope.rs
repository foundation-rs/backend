use std::sync::Arc;
use actix_web::{get, web, Scope, Responder, HttpResponse};
use serde::Deserialize;

use crate::application::{ApplicationState, query};
use actix_web::http::header::ContentType;
use std::collections::{HashMap, HashSet};

// group of endpoints for api
pub fn api_scope() -> Scope {
    web::scope("/api")
        .service(table_query_by_pk)
        .service(table_query_by_params)
}

#[get("/schemas/{schema}/{table}/{pk}")]
async fn table_query_by_pk(path: web::Path<(String,String,String)>, data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let (schema_name,table_name, primary_key) = path.into_inner();
    let metainfo = data.metainfo.read().unwrap();

    if let Some(info) = metainfo.schemas.get(schema_name.as_str()) {
        if let Some(info) = info.tables.get(table_name.as_str()) {
            let query = query::DynamicQuery::create_from_pk(&schema_name, info, primary_key);
            return match query {
                Ok(query) => {
                    let result = query.fetch_one();
                    match result {
                        Ok(r) => HttpResponse::Ok().set(ContentType::json()).body(r),
                        Err(err) => HttpResponse::InternalServerError().body(err)
                    }
                },
                Err(err) => HttpResponse::BadRequest().body(err)
            };
        }
    };

    HttpResponse::NotFound().finish()
}

// for limit, offset etc, see: https://oracletutorial.com/oracle-basics/oracle-fetch

#[derive(Deserialize)]
struct QueryParams {
    q:      String,
    limit:  Option<u16>,
    offset: Option<u16>,
    order:  Option<String>,
}

#[get("/schemas/{schema}/{table}/")]
async fn table_query_by_params(path: web::Path<(String,String)>, req: web::Query<QueryParams>, data: web::Data<Arc<ApplicationState>>) -> impl Responder {
    let (schema_name,table_name) = path.into_inner();
    let metainfo = data.metainfo.read().unwrap();

    if let Some(info) = metainfo.schemas.get(schema_name.as_str()) {
        if let Some(info) = info.tables.get(table_name.as_str()) {
            println!("{}.{}; q: {}", schema_name, table_name, req.q);

            let q: serde_json::error::Result<HashMap<String,String>> = serde_json::from_str(&req.q);
            return match q {
                Ok(paremeters) => {
                    let query = query::DynamicQuery::create_from_params(&schema_name, info, paremeters, req.limit, req.offset);
                    return match query {
                        Ok(query) => {
                            let result = query.fetch_many();
                            match result {
                                Ok(r) => HttpResponse::Ok().set(ContentType::json()).body(r),
                                Err(err) => HttpResponse::InternalServerError().body(err)
                            }
                        },
                        Err(err) => HttpResponse::BadRequest().body(err)
                    };
                },
                Err(err) => HttpResponse::BadRequest().body(format!("Invalid query format: {}", err))
            };
        }
    };

    HttpResponse::NotFound().finish()
}
