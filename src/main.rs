mod config;

// TODO: use xml for config
// SEE: https://github.com/tafia/quick-xml

use oracle;
use oracle_derive::Query;

fn main() -> Result<(), &'static str> {
    let ref conf = config::load("config.json")?;

    let ref cc = conf.connection;

    let conn = oracle::connect(&cc.url, &cc.user, &cc.pw)
        .map_err(|err|"Can not connect to Oracle")?;

    let tables = load(&conn, &conf.excludes)
        .map_err(|err| "Can not read metainfo abaut oracle tables")?;
    for t in &tables {
        println!("t {}.{}", t.owner, t.table_name);
    }
    println!("total tables: {}", tables.len());

    Ok(())
}

#[derive(Query)]
pub struct OraTable {
    owner: String,
    table_name: String
}

pub fn load(conn: &oracle::Connection, excludes: &Vec<String>) -> Result<Vec<OraTable>,oracle::OracleError> {
    use std::ops::Add;

    let quoted_excludes: Vec<String> = excludes.iter().map(|s| format!("'{}'", s) ).collect();
    let sql = format!(
        "SELECT OWNER, TABLE_NAME FROM SYS.ALL_TABLES WHERE OWNER NOT IN ( {} ) ORDER BY OWNER, TABLE_NAME",
            &quoted_excludes.join(","));

    let mut result = Vec::with_capacity(8000);
    let mut query = conn.query::<OraTable>(&sql)?;

    for v in query.fetch_iter()? {
        if let Ok(v) = v {
            result.push(v);
        };
    }

    Ok(result)
}

#[derive(Query)]
pub struct TestingTuple (i32, String, String);
