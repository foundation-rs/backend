use oracle;
use oracle::QueryIterator;
use oracle_derive::ResultsProvider;

use super::types::*;

#[derive(ResultsProvider)]
pub struct OraTable {
    pub owner:      String,
    pub table_name: String,
    pub table_type: String,
    pub num_rows:   i32,
    pub temporary:  String,
}

pub type OraTablesIterator<'iter, 'conn> = QueryIterator<'iter, 'conn, (), OraTable, 5000>;

#[derive(ResultsProvider)]
pub struct OraTableColumn {
    pub owner:          String,
    pub table_name:     String,
    pub column_name:    String,
    pub data_type:      String,
    pub data_length:    u16,
    pub data_precision: u16,
    pub data_scale:     u16,
    pub nullable:       String
}

pub type OraColumnsIterator<'iter, 'conn> = QueryIterator<'iter, 'conn, (), OraTableColumn, 10000>;

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

    let query = conn.prepare(&sql)?.query()?;
    query.fetch_iter(())
}

pub fn fetch_columns<'iter, 'conn: 'iter>(conn: &'conn oracle::Connection, excludes: &str) -> oracle::OracleResult<OraColumnsIterator<'iter, 'conn>> {
    let sql = format!(
        "SELECT OWNER, TABLE_NAME, COLUMN_NAME, DATA_TYPE, DATA_LENGTH, DATA_PRECISION, DATA_SCALE, NULLABLE \
        FROM SYS.ALL_TAB_COLUMNS WHERE OWNER NOT IN ( {} ) ORDER BY OWNER, TABLE_NAME, COLUMN_ID", excludes);

    let query = conn.prepare(&sql)?.query()?;
    query.fetch_iter(())
}
