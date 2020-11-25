use std::sync::Arc;
use actix_web::{Scope, web, Responder, HttpResponse};

use crate::application::ApplicationState;
use oracle;

// group of endpoints for api
pub fn api_scope() -> Scope {
    web::scope("/api")
        .service(table_query_by_pk)
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

struct DynamicQuery {

}

impl oracle::ResultsProvider<DynamicQuery> for DynamicQuery {

    fn sql_descriptors(&self) -> Vec<oracle::TypeDescriptor> {
        unimplemented!()
    }

    fn gen_result(&self, rs: oracle::ResultSet) -> DynamicQuery {
        unimplemented!()
    }
}

impl oracle::ParamsProvider<DynamicQuery> for DynamicQuery {
    fn members(&self) -> Vec<oracle::Member> {
        unimplemented!()
    }

    fn project_values(&self, params: &T, projecton: &mut oracle::ParamsProjection) {
        unimplemented!()
    }
}

