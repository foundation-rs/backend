
use std::collections::HashMap;

use oracle;
use oracle_derive::Query;

pub struct MetaInfo {
    pub schemas:  HashMap<String,Schema>,
}

pub struct Schema {
    pub tables:  HashMap<String,Table>,
}

#[derive(Debug, PartialEq)]
pub enum TableType { Table, View }

pub struct Table {
    pub table_type: TableType,
    pub num_rows:    i32,
    pub columns:     Vec<OraTableColumn>,
}

#[derive(Query)]
pub struct OraTable {
    owner: String,
    table_name: String,
    table_type: String,
    num_rows: i32
}

#[derive(Query)]
pub struct OraTableColumn {
    column_id: i16,
    owner: String,
    table_name: String,
    column_name: String,
    data_type: String,
    data_length: i16,
    nullable: String
}

impl MetaInfo {
    pub fn new(conn: &oracle::Connection, excludes: &Vec<String>) -> oracle::OracleResult<MetaInfo> {
        let quoted_excludes: Vec<String> = excludes.iter().map(|s| format!("'{}'", s) ).collect();
        let joined_excludes = &quoted_excludes.join(",");

        let schemas = MetaInfo::load(conn, &joined_excludes)?;
        Ok( MetaInfo { schemas })
    }

    fn load(conn: &oracle::Connection, excludes: &str)-> oracle::OracleResult<HashMap<String,Schema>> {
        let sql = format!(
            "SELECT OWNER, TABLE_NAME, TABLE_TYPE, NUM_ROWS FROM (
            SELECT OWNER, TABLE_NAME, 'TABLE' AS TABLE_TYPE, NUM_ROWS FROM SYS.ALL_TABLES
            UNION
            SELECT OWNER, VIEW_NAME, 'VIEW' AS TABLE_TYPE, 0 FROM SYS.ALL_VIEWS 
            ) WHERE OWNER NOT IN ( {} )
            ORDER BY OWNER, TABLE_NAME", excludes);

        let sql_columns = format!(
            "SELECT COLUMN_ID, OWNER, TABLE_NAME, COLUMN_NAME, DATA_TYPE, DATA_LENGTH, NULLABLE \
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
        let mut previous_column: Option<OraTableColumn> = None;

        for v in query.fetch_iter(())? {
            if let Ok(v) = v {
                let ref owner = v.owner;
                let table_name = v.table_name;
                let num_rows = v.num_rows;

                let table_type = match v.table_type.as_ref() {
                    "TABLE" => TableType::Table,
                    "VIEW" => TableType::View,
                    _ => unreachable!()
                };

                // println!("{}.{}; {:?}; rows: {}", owner, &table_name, &v.table_type, &num_rows);

                // iterate over columns_iterator and construct vector for current table
                let mut columns = Vec::with_capacity(100);

                if let Some(c) = previous_column.take() {
                    columns.push(c);
                };

                for c in &mut columns_iterator {
                    if let Ok(c) = c {
                        let ref c_owner = c.owner;
                        let ref c_table_name = c.table_name;

                        // let nn = if c.nullable == "Y" { "" } else { "NOT NULL" };
                        // print!("     {}::{}.{} {}({}) {}", c_owner, c_table_name, &c.column_name, &c.data_type, c.data_length, nn);
    
                        if c_owner == owner && c_table_name == &table_name {
                            // println!("   push");
                            columns.push(c);
                        } else {
                            // transfer column to next iteration
                            // println!("   transfer...");
                            previous_column = Some(c);
                            break;
                        }
                    }
                }

                let schema = result.entry(v.owner).or_insert_with(|| Schema { tables: HashMap::with_capacity(100) });
                schema.tables.entry(table_name).or_insert(Table { table_type, num_rows, columns });
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

