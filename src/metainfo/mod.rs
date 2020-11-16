
use std::collections::HashMap;
use std::rc::Rc;

use oracle;
use oracle_derive::ResultsProvider;
use crate::utils;

pub struct MetaInfo {
    pub schemas:  HashMap<Rc<String>,SchemaInfo>,
}

pub struct SchemaInfo {
    pub name:    Rc<String>,
    pub tables:  HashMap<Rc<String>,TableInfo>,
}

pub struct TableInfo {
    pub name:        Rc<String>,
    pub is_view:     bool,
    pub temporary:   bool,
    pub num_rows:    i32,
    pub columns:     Vec<ColumnInfo>,
    pub primary_key: Option<PrimaryKey>,
    pub indexes:     Vec<Index>
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColumnType {
    Int16, Int32, Int64, Float64, Varchar, DateTime, Blob, Clob, Long, Unsupported
}

pub struct ColumnInfo {
    pub name:           String,
    pub col_type:       ColumnType,
    pub col_type_name:  String,
    pub oci_data_type:  u16,
    pub col_len:        u16,
    pub nullable:       bool,
    pub data_precision: u16,
    pub data_scale:     u16,
    pub buffer_len:     usize
}

pub struct PrimaryKey {
    pub name:    String,
    pub columns: Vec<String>
}

pub struct Index {
    pub name:    String,
    pub unique:  bool,
    pub columns: Vec<IndexColumn>
}

pub struct IndexColumn {
    pub name: String,
    pub desc: bool
}

#[derive(ResultsProvider)]
struct OraTable {
    owner:      String,
    table_name: String,
    table_type: String,
    num_rows:   i32,
    temporary:  String,
}

#[derive(ResultsProvider)]
struct OraTableColumn {
    owner:          String,
    table_name:     String,
    column_name:    String,
    data_type:      String,
    data_length:    u16,
    data_precision: u16,
    data_scale:     u16,
    nullable:       String
}

impl MetaInfo {
    pub fn new(conn: &oracle::Connection, excludes: &Vec<String>) -> oracle::OracleResult<MetaInfo> {
        let quoted_excludes: Vec<String> = excludes.iter().map(|s| format!("'{}'", s) ).collect();
        let joined_excludes = &quoted_excludes.join(",");

        let schemas = MetaInfo::load(conn, &joined_excludes)?;
        Ok( MetaInfo { schemas })
    }

    fn load(conn: &oracle::Connection, excludes: &str)-> oracle::OracleResult<HashMap<Rc<String>,SchemaInfo>> {
        let sql = format!(
            "SELECT OWNER, TABLE_NAME, TABLE_TYPE, NUM_ROWS, TEMPORARY FROM (
            SELECT OWNER, TABLE_NAME, 'TABLE' AS TABLE_TYPE, NUM_ROWS, TEMPORARY
            FROM SYS.ALL_TABLES
            UNION
            SELECT OWNER, VIEW_NAME, 'VIEW' AS TABLE_TYPE, 0, 'N'
            FROM SYS.ALL_VIEWS 
            ) WHERE OWNER NOT IN ( {} )
            ORDER BY OWNER, TABLE_NAME", excludes);

        let sql_columns = format!(
            "SELECT OWNER, TABLE_NAME, COLUMN_NAME, DATA_TYPE, DATA_LENGTH, DATA_PRECISION, DATA_SCALE, NULLABLE \
            FROM SYS.ALL_TAB_COLUMNS WHERE OWNER NOT IN ( {} ) ORDER BY OWNER, TABLE_NAME, COLUMN_ID", excludes);
    
        // tables and columns queries/iterators are sorted by owner, table_name and synchronized

        let mut result = HashMap::with_capacity(5000);

        let query = conn
            .prepare(&sql)?
            .query_many::<OraTable, 1000>()?;

        let columns_query = conn
            .prepare(&sql_columns)?
            .query_many::<OraTableColumn, 1000>()?;

        let mut columns_iterator = columns_query.fetch_iter(())?;

        let mut current_schema = None;
        let mut previous_column: Option<OraTableColumn> = None;

        for v in query.fetch_iter(())? {
            if let Ok(v) = v {
                let ref owner = v.owner;
                let table_name = v.table_name.clone();
                let num_rows = v.num_rows;

                let is_view = v.table_type == "VIEW";
                let temporary = v.temporary == "Y";

                // println!("{}.{}; {:?}; rows: {}", owner, &table_name, &v.table_type, &num_rows);

                // iterate over columns_iterator and construct vector for current table
                let mut columns = Vec::with_capacity(100);

                if let Some(c) = previous_column.take() {
                    columns.push(c.into());
                };

                for c in &mut columns_iterator {
                    if let Ok(c) = c {
                        let ref c_owner = c.owner;
                        let ref c_table_name = c.table_name;

                        // print!("     {}::{}.{} {}({})", c_owner, c_table_name, &c.column_name, &c.data_type, c.data_length);
    
                        if c_owner == owner && c_table_name == &table_name {
                            // println!("   push");
                            columns.push(c.into());
                        } else {
                            // transfer column to next iteration
                            // println!("   transfer...");
                            previous_column = Some(c);
                            break;
                        }
                    }
                }

                let (schema,old_schema) = 
                    utils::get_or_insert_with_condition(
                        &mut current_schema, 
                        || SchemaInfo { name: Rc::new(v.owner.clone()), tables: HashMap::with_capacity(100) }, 
                        |s| s.name.as_ref() == &v.owner);

                if let Some(old_schema) = old_schema {
                    result.insert(old_schema.name.clone(), old_schema);
                };
               
                let table_name = Rc::new(table_name);
                schema.tables.insert(table_name.clone(), TableInfo { name: table_name, is_view, temporary, num_rows, columns, primary_key: None, indexes: Vec::new() });
                
                // let schema = result.entry(v.owner).or_insert_with(|| Schema { tables: HashMap::with_capacity(100) });
                // schema.tables.entry(table_name.clone()).or_insert(TableInfo { name: table_name, is_view, temporary, num_rows, columns, primary_key: None, indexes: Vec::new() });
            };
        }            
    
        Ok(result)
    }

    
    /*
    fn load(conn: &oracle::Connection, excludes: &Vec<String>) -> oracle::OracleResult<Vec<OraTable>> {
        let quoted_excludes: Vec<String> = excludes.iter().map(|s| format!("'{}'", s) ).collect();
        let sql = format!(
            "SELECT OWNER, TABLE_NAME, NUM_ROWS FROM SYS.ALL_TABLES WHERE OWNER NOT IN ( {} ) ORDER BY OWNER, TABLE_NAME",
                &quoted_excludes.join(","));
    
        let sql_cols =
            "SELECT COLUMN_ID, OWNER, TABLE_NAME, COLUMN_NAME, DATA_TYPE, DATA_LENGTH, NULLABLE \
            FROM SYS.ALL_TAB_COLUMNS WHERE OWNER = :own AND TABLE_NAME = :nm";
    
        let mut result = Vec::with_capacity(8000);
    
        let query = conn
            .prepare(&sql)?
            .query_many::<OraTable, 1000>()?;
    
        let colmns_query = conn
            .prepare(&sql_cols)?
            .query_many::<OraTableColumn, 100>()?;
    
        let mut columns_cnt = 0;
    
        for v in query.fetch_iter(())? {
            if let Ok(v) = v {
                println!("{}.{}; rows: {}", &v.owner, &v.table_name, &v.num_rows);
                // let params = OraTableColumnParams { own: &v.owner, nm: &v.table_name };
                let columns = colmns_query.fetch_list((v.owner.as_ref(), v.table_name.as_ref()))?;
                for c in columns {
                    let nn = if c.nullable == "Y" { "" } else { "NOT NULL" };
                    println!("   c {} {}({}) {}", c.column_name, c.data_type, c.data_length, nn);
                    columns_cnt +=1;
                }
                result.push(v);
            };
        }
    
        println!("total columns: {}", columns_cnt);
    
        Ok(result)
    }
    */
}

impl From<OraTableColumn> for ColumnInfo {
    fn from(v: OraTableColumn) -> ColumnInfo {
        use std::mem::size_of;
    
        let name = v.column_name;
        let nullable = v.nullable == "Y";
        let data_scale = v.data_scale;
        let data_precision = v.data_precision;
        let col_len = v.data_length;

        let mut col_type_name = v.data_type;

        let (col_type, oci_data_type, buffer_len) = {
            let ctn: &str = &col_type_name.clone();
            match ctn {
                "CHAR" | "VARCHAR2" => {
                    // SQLT_CHR
                    (ColumnType::Varchar, 1, col_len as usize)
                },
                "LONG" => {
                    // SQLT_CHR
                    (ColumnType::Long, 1, 4000)
                },
                "DATE" => {
                    // SQLT_DAT
                    (ColumnType::DateTime, 1, 12)
                },
                "CLOB" => {
                    // SQLT_CLOB
                    (ColumnType::Clob, 112, 0)
                },
                "BLOB" => {
                    // SQLT_BLOB
                    (ColumnType::Blob, 113, 0)
                },
                "NUMBER" => {
                    if data_scale == 0 {
                        if data_precision == 0 || data_precision > 7 {
                            if data_precision == 0 {
                                col_type_name = "INTEGER".to_string();
                            }
                            // SQLT_NUM
                            (ColumnType::Int64, 2, size_of::<i64>())
                        } else if data_precision > 4 {
                            // SQLT_NUM
                            (ColumnType::Int32, 2, size_of::<i32>())
                        } else {
                            // SQLT_NUM
                            (ColumnType::Int16, 2, size_of::<i16>())
                        }
                    } else {
                        // SQLT_NUM
                        (ColumnType::Float64, 2, size_of::<f64>())
                    }
                },
                _ => {
                    // Unsupported
                    (ColumnType::Unsupported, 0, 0)
                }
            }
        };

        ColumnInfo { name, col_type, col_type_name, oci_data_type, col_len, nullable, data_precision, data_scale, buffer_len }
    }
}