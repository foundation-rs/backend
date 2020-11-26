use std::sync::Arc;
use actix_web::{get, web, Scope, Responder, HttpResponse};

use crate::application::ApplicationState;
use crate::metainfo as mi;
use oracle;
use crate::metainfo::{TableInfo, PrimaryKey};

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
                    let query = DynamicQuery::new(&info.name, info, pk);
                    let select = query.generate_sql();
                    HttpResponse::Ok().body(select)
                },
                _ => HttpResponse::BadRequest().finish()
            };
        }
    };

    HttpResponse::NotFound().finish()
}

struct DynamicQuery {
    table_name:    String,
    column_names:  Vec<String>,
    column_types:  Vec<oracle::TypeDescriptor>,
    param_columns: Vec<usize>
}

impl DynamicQuery {
    pub fn new(schema_name: &str, table_info: &mi::TableInfo, pk: &mi::PrimaryKey) -> DynamicQuery {
        let table_name = format!("{}.{}", schema_name, table_info.name.as_str());

        let pk_coloumn_name = unsafe { pk.columns.get_unchecked(0) };

        let columns_count = table_info.columns.len();
        let mut column_names = Vec::with_capacity(columns_count);
        let mut column_types = Vec::with_capacity(columns_count);
        let mut param_columns = Vec::new();

        for i in 0..columns_count {
            let column = unsafe { table_info.columns.get_unchecked(i) };
            column_names.push(column.name.clone());
            column_types.push(oracle::TypeDescriptor::new(column.oci_data_type, column.buffer_len));
            if &column.name == pk_coloumn_name {
                param_columns.push(i);
            }
        }

        DynamicQuery { table_name, column_names, column_types, param_columns }
    }

    pub fn generate_sql(&self) -> String {
        let joined_result_columns = self.column_names.join(",");

        let param_columns: Vec<String> = self.param_columns.iter().map(|idx|unsafe { self.column_names.get_unchecked(*idx) }.clone()).collect();

        let where_clause = if param_columns.len() == 1 {
            format!("{} = :1", unsafe { param_columns.get_unchecked(0) })
        } else {
            let enumerated_param_columns: Vec<String> =
                param_columns.iter().enumerate().map(|(idx,name)|format!("{} = {}", name, idx+1)).collect();
            enumerated_param_columns.join(" AND ")
        };
        format!("SELECT {} FROM {} WHERE {}", joined_result_columns, self.table_name, where_clause)
    }
}

impl oracle::ResultsProvider<DynamicQuery> for DynamicQuery {
    fn sql_descriptors(&self) -> Vec<oracle::TypeDescriptor> {
        self.column_types.clone()
    }

    fn gen_result(&self, rs: oracle::ResultSet) -> DynamicQuery {
        unimplemented!()
    }
}

impl oracle::ParamsProvider<DynamicQuery> for DynamicQuery {
    fn members(&self) -> Vec<oracle::Member> {
        unimplemented!()
    }

    fn project_values(&self, params: &DynamicQuery, projecton: &mut oracle::ParamsProjection) {
        unimplemented!()
    }
}

