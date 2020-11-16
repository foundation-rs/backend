use oracle;
use oracle_derive::ResultsProvider;

pub use super::types::*;
use oracle::QueryIterator;

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

pub type OraColumnsIterator<'iter, 'conn: 'iter> = QueryIterator<'iter, 'conn, (), OraTableColumn, 1000>;

pub fn fetch_columns<'iter, 'conn: 'iter>(conn: &'conn oracle::Connection, excludes: &str) -> oracle::OracleResult<OraColumnsIterator<'iter, 'conn>> {
    let sql = format!(
        "SELECT OWNER, TABLE_NAME, COLUMN_NAME, DATA_TYPE, DATA_LENGTH, DATA_PRECISION, DATA_SCALE, NULLABLE \
        FROM SYS.ALL_TAB_COLUMNS WHERE OWNER NOT IN ( {} ) ORDER BY OWNER, TABLE_NAME, COLUMN_ID", excludes);

    let query = conn.prepare(&sql)?.query_many()?;
    query.fetch_iter(())
}
