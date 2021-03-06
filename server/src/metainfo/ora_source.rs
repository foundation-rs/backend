use oracle;
use oracle::QueryIterator;
use oracle_derive::SQLResults;

#[derive(SQLResults)]
pub struct OraTable {
    pub owner:      String,
    pub table_name: String,
    #[col_size=8]
    pub table_type: String,
    pub num_rows:   i32,
    #[col_size=2]
    pub temporary:  String,
}

pub type TablesIterator<'iter, 'conn> = QueryIterator<'iter, 'conn, (), OraTable>;

#[derive(SQLResults)]
pub struct OraTableColumn {
    pub owner:          String,
    pub table_name:     String,
    pub column_name:    String,
    pub data_type:      String,
    pub data_length:    u16,
    pub data_precision: u16,
    pub data_scale:     u16,
    #[col_size=2]
    pub nullable:       String
}

pub type ColumnsIterator<'iter, 'conn> = QueryIterator<'iter, 'conn, (), OraTableColumn>;

#[derive(SQLResults)]
pub struct OraTablePrimaryKeyColumn {
    pub owner:           String,
    pub table_name:      String,
    pub constraint_name: String,
    pub column_name:     String
}

pub type PrimaryKeyColumnsIterator<'iter, 'conn> = QueryIterator<'iter, 'conn, (), OraTablePrimaryKeyColumn>;

#[derive(SQLResults)]
pub struct OraTableIndexColumn {
    pub owner:       String,
    pub table_name:  String,
    pub index_name:  String,
    #[col_size=10]
    pub uniqueness:  String,
    pub column_name: String,
    #[col_size=4]
    pub descend:     String
}

pub type IndexColumnsIterator<'iter, 'conn> = QueryIterator<'iter, 'conn, (), OraTableIndexColumn>;

pub fn fetch_tables<'iter, 'conn: 'iter>(conn: &'conn oracle::Connection, excludes: &str) -> oracle::OracleResult<TablesIterator<'iter, 'conn>> {
    let sql = format!(
        "SELECT OWNER, TABLE_NAME, TABLE_TYPE, NUM_ROWS, TEMPORARY FROM (
        SELECT OWNER, TABLE_NAME, 'TABLE' AS TABLE_TYPE, NUM_ROWS, TEMPORARY
        FROM SYS.ALL_TABLES
        UNION
        SELECT OWNER, VIEW_NAME, 'VIEW' AS TABLE_TYPE, 0, 'N'
        FROM SYS.ALL_VIEWS
        ) WHERE OWNER NOT IN ( {} )
        ORDER BY OWNER, TABLE_NAME"
        ,excludes
    );

    let query = conn.prepare(&sql)?.query_many(5000)?;
    query.fetch_iter(())
}

pub fn fetch_columns<'iter, 'conn: 'iter>(conn: &'conn oracle::Connection, excludes: &str) -> oracle::OracleResult<ColumnsIterator<'iter, 'conn>> {
    let sql = format!(
        "SELECT OWNER, TABLE_NAME, COLUMN_NAME, DATA_TYPE, DATA_LENGTH, DATA_PRECISION, DATA_SCALE, NULLABLE \
        FROM SYS.ALL_TAB_COLUMNS WHERE OWNER NOT IN ( {} ) ORDER BY OWNER, TABLE_NAME, COLUMN_ID"
        ,excludes
    );

    let query = conn.prepare(&sql)?.query_many(50000)?;
    query.fetch_iter(())
}

pub fn fetch_primary_keys<'iter, 'conn: 'iter>(conn: &'conn oracle::Connection, excludes: &str) -> oracle::OracleResult<PrimaryKeyColumnsIterator<'iter, 'conn>> {
    let sql = format!(
        "SELECT C.OWNER, C.TABLE_NAME, C.CONSTRAINT_NAME, CC.COLUMN_NAME \
        FROM SYS.ALL_CONSTRAINTS C \
        JOIN SYS.ALL_CONS_COLUMNS CC ON C.OWNER = CC.OWNER AND C.TABLE_NAME = CC.TABLE_NAME AND C.CONSTRAINT_NAME = CC.CONSTRAINT_NAME
        WHERE C.OWNER NOT IN ( {} ) AND C.CONSTRAINT_TYPE = 'P' AND C.STATUS = 'ENABLED'
        ORDER BY C.OWNER, C.TABLE_NAME, C.CONSTRAINT_NAME, CC.POSITION"
        ,excludes
    );

    let query = conn.prepare(&sql)?.query_many(1000)?;
    query.fetch_iter(())
}

pub fn fetch_indexes<'iter, 'conn: 'iter>(conn: &'conn oracle::Connection, excludes: &str) -> oracle::OracleResult<IndexColumnsIterator<'iter, 'conn>> {
    let sql = format!(
        "SELECT C.TABLE_OWNER, C.TABLE_NAME, C.INDEX_NAME, C.UNIQUENESS, CC.COLUMN_NAME, CC.DESCEND \
        FROM SYS.ALL_INDEXES C \
        JOIN SYS.ALL_IND_COLUMNS CC ON C.TABLE_OWNER = CC.INDEX_OWNER AND C.INDEX_NAME = CC.INDEX_NAME
        WHERE C.OWNER NOT IN ( {} ) AND C.STATUS = 'VALID'
        ORDER BY C.TABLE_OWNER, C.TABLE_NAME, C.INDEX_NAME, CC.COLUMN_POSITION"
        ,excludes
    );

    let query = conn.prepare(&sql)?.query_many(1000)?;
    query.fetch_iter(())
}
