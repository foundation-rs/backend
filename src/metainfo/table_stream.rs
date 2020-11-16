use oracle;
use oracle_derive::ResultsProvider;

pub use super::types::*;
use oracle::QueryIterator;

#[derive(ResultsProvider)]
pub struct OraTable {
    pub owner:      String,
    pub table_name: String,
    pub table_type: String,
    pub num_rows:   i32,
    pub temporary:  String,
}

pub struct OraTableStreamSource<'iter, 'conn: 'iter> {
    pub iterator: QueryIterator<'iter, 'conn, (), OraTable, 1000>
}

impl <'iter, 'conn: 'iter> OraTableStreamSource<'iter, 'conn> {
    pub fn stream(conn: &'conn oracle::Connection, excludes: &str) -> oracle::OracleResult<OraTableStreamSource<'iter, 'conn>> {
        let sql = format!(
            "SELECT OWNER, TABLE_NAME, TABLE_TYPE, NUM_ROWS, TEMPORARY FROM (
            SELECT OWNER, TABLE_NAME, 'TABLE' AS TABLE_TYPE, NUM_ROWS, TEMPORARY
            FROM SYS.ALL_TABLES
            UNION
            SELECT OWNER, VIEW_NAME, 'VIEW' AS TABLE_TYPE, 0, 'N'
            FROM SYS.ALL_VIEWS
            ) WHERE OWNER NOT IN ( {} )
            ORDER BY OWNER, TABLE_NAME", excludes);

        let query = conn.prepare(&sql)?.query_many()?;
        let iterator = query.fetch_iter(())?;

        Ok(OraTableStreamSource { iterator } )
    }
}
