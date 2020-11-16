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

pub type OraTablesIterator<'iter, 'conn: 'iter> = QueryIterator<'iter, 'conn, (), OraTable, 1000>;

pub fn fetch_tables<'iter, 'conn: 'iter>(conn: &'conn oracle::Connection, excludes: &str) -> oracle::OracleResult<OraTablesIterator<'iter, 'conn>> {
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
    query.fetch_iter(())
}

