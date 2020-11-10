mod config;

// TODO: use xml for config
// SEE: https://github.com/tafia/quick-xml

use oracle;
use oracle_derive::{Params,Query};
use oracle::ValueProjector;

fn main() -> Result<(), String> {
    let start = chrono::offset::Local::now();

    let ref conf = config::load("config.json")?;
    let ref cc = conf.connection;

    let conn = oracle::connect(&cc.url, &cc.user, &cc.pw)
        .map_err(|err| format!("Can not connect to Oracle: {}", err))?;

    let tables = load(&conn, &conf.excludes)
        .map_err(|err| format!("Can not read metainfo about oracle tables: {}", err))?;
    for t in &tables {
        println!("t {}.{}; rows: {}", t.owner, t.table_name, t.num_rows);
    }
    println!("total tables: {}", tables.len());

    let end = chrono::offset::Local::now();
    let duration = end - start;

    println!("ELAPSED: {} seconds, {} milliseconds", duration.num_seconds(), duration.num_milliseconds());

    Ok(())
}

#[derive(Query)]
pub struct OraTable {
    owner: String,
    table_name: String,
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

// TODO: convert String to &'a str
// TODO: proper lifetimes
// pub struct OraTableColumnParams (String, String);
#[derive(Params)]
pub struct OraTableColumnParams<'a> {
    own: &'a str,
    nm: &'a str,
}

pub fn load(conn: &oracle::Connection, excludes: &Vec<String>) -> Result<Vec<OraTable>,oracle::OracleError> {
    use std::ops::Add;

    let quoted_excludes: Vec<String> = excludes.iter().map(|s| format!("'{}'", s) ).collect();
    let sql = format!(
        "SELECT OWNER, TABLE_NAME, NUM_ROWS FROM SYS.ALL_TABLES WHERE OWNER NOT IN ( {} ) ORDER BY OWNER, TABLE_NAME",
            &quoted_excludes.join(","));

    /*
    let sql_cols =
        "SELECT COLUMN_ID, OWNER, TABLE_NAME, COLUMN_NAME, DATA_TYPE, DATA_LENGTH, NULLABLE \
        FROM SYS.ALL_TAB_COLUMNS WHERE OWNER = :1 AND TABLE_NAME = :2";
    */
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

    for v in query.fetch_iter(&())? {
        if let Ok(v) = v {
            // let params = OraTableColumnParams { own: &v.owner, nm: &v.table_name };
            let columns = colmns_query.fetch_list(&(v.owner.as_ref(), v.table_name.as_ref()))?;
            for c in columns {
                // println!("   c {} {}", c.column_name, c.data_type);
                columns_cnt +=1;
            }
            result.push(v);
        };
    }

    println!("total columns: {}", columns_cnt);

    Ok(result)
}
