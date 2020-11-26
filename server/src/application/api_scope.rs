use std::sync::Arc;
use actix_web::{get, web, Scope, Responder, HttpResponse};

use oracle::{self, ValueProjector};

use crate::application::ApplicationState;
use crate::{metainfo as mi, datasource};
use std::ffi::OsString;
use actix_web::http::header::ContentType;

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
                    let result = query.execute_query(primary_key);
                    match result {
                        Ok(r) => HttpResponse::Ok().set(ContentType::json()).body(r),
                        Err(err) => HttpResponse::InternalServerError().body(err)
                    }
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
    column_types:  Vec<mi::ColumnType>,
    column_type_descriptors:  Vec<oracle::TypeDescriptor>,
    param_columns: Vec<usize>
}

impl DynamicQuery {
    // TODO: check validity of query, pk must be int or String
    // TODO: if pk is int, try parse String to int
    pub fn new(schema_name: &str, table_info: &mi::TableInfo, pk: &mi::PrimaryKey) -> DynamicQuery {
        let table_name = format!("{}.{}", schema_name, table_info.name.as_str());

        let pk_coloumn_name = unsafe { pk.columns.get_unchecked(0) };

        let columns_count = table_info.columns.len();
        let mut column_names = Vec::with_capacity(columns_count);
        let mut column_types = Vec::with_capacity(columns_count);
        let mut column_type_descriptors = Vec::with_capacity(columns_count);
        let mut param_columns = Vec::new();

        for i in 0..columns_count {
            let column = unsafe { table_info.columns.get_unchecked(i) };
            column_names.push(column.name.clone());
            column_types.push(column.col_type);
            column_type_descriptors.push(oracle::TypeDescriptor::new(column.oci_data_type, column.buffer_len));
            if &column.name == pk_coloumn_name {
                param_columns.push(i);
            }
        }

        DynamicQuery { table_name, column_names, column_types, column_type_descriptors, param_columns }
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

    /// execyte a query and generate JSON result
    pub fn execute_query(self, pk: String) -> Result<String,String> {
        let conn = datasource::get_connection()
            .map_err(|err|format!("Can not connect to oracle: {}", err))?;

        let sql = self.generate_sql();
        let stmt = conn.prepare(&sql)
            .map_err(|err|format!("Can not prepare statement: {}", err))?;

        let query = stmt.query_dynamic(Box::new(self), 1)
            .map_err(|err|format!("Can not create query from statement: {}", err))?;

        let result = query.fetch_one(pk)
            .map_err(|err|format!("Can not fetch row by pk: {}", err))?;

        Ok( format!("[{}]", result) )
    }
}

impl oracle::ResultsProvider<String> for DynamicQuery {
    fn sql_descriptors(&self) -> Vec<oracle::TypeDescriptor> {
        self.column_type_descriptors.clone()
    }

    fn gen_result(&self, rs: oracle::ResultSet) -> String {
        let mut results = Vec::with_capacity(self.column_names.len());

        for (idx,t) in self.column_types.iter().enumerate() {
            // println!("col {} has type {:?}", idx, t);

            let value = unsafe { rs.get_unchecked(idx) }.to_owned();
            let value = match t {
                mi::ColumnType::Varchar => {
                    let v: String = value.into();
                    format!("\"{}\"",v)
                },
                mi::ColumnType::Int16 => {
                    let v: i16 = value.into();
                    v.to_string()
                },
                mi::ColumnType::Int32 => {
                    let v: i32 = value.into();
                    v.to_string()
                },
                mi::ColumnType::Int64 => {
                    let v: i64 = value.into();
                    v.to_string()
                },
                mi::ColumnType::Float64 => {
                    let v: f64 = value.into();
                    v.to_string()
                },
                mi::ColumnType::DateTime => {
                    let v: oracle::SqlDateTime = value.into();
                    format!("\"{}\"", v.to_rfc3339())
                }
                _ => "\"not-implemented\"".to_string()
            };
            let name = unsafe { self.column_names.get_unchecked(idx) };
            results.push(format!("\"{}\":{}", name, value));
        }

        format!("{{ {} }}", results.join(","))
    }
}

// TODO: may be type of params must be Special Struct with checked and parsed value, may be union

impl oracle::ParamsProvider<String> for DynamicQuery {
    fn members(&self) -> Vec<oracle::Member> {
        self.param_columns.iter()
            .map(|idx| {
                let td = unsafe { self.column_type_descriptors.get_unchecked(*idx) }.clone();
                oracle::Member::new(td, oracle::Identifier::Unnamed)
            })
            .collect()
    }

    // currently, string representation of primary key
    fn project_values(&self, params: &String, projecton: &mut oracle::ParamsProjection) {
        let param_column_idx = unsafe { self.param_columns.get_unchecked(0) };
        let param_column_type = unsafe { self.column_types.get_unchecked(*param_column_idx) };

        let p = unsafe { projecton.get_unchecked_mut(0) };

        match param_column_type {
            mi::ColumnType::Int16 => {
                let val: i16 = params.parse().unwrap();
                val.project_value(p);
            },
            mi::ColumnType::Int32 => {
                let val: i32 = params.parse().unwrap();
                val.project_value(p);
            },
            mi::ColumnType::Int64 => {
                let val: i64 = params.parse().unwrap();
                val.project_value(p);
            },
            mi::ColumnType::Varchar => {
                &params.project_value(p);
            },
            _ => {}
        };
    }
}

