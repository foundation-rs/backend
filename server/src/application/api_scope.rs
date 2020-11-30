use std::sync::Arc;
use actix_web::{get, web, Scope, Responder, HttpResponse};

use crate::application::{ApplicationState, query};
use actix_web::http::header::ContentType;

// group of endpoints for api
pub fn api_scope() -> Scope {
    web::scope("/api")
        .service(table_query_by_pk)
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
