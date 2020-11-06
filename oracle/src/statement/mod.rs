pub mod params;
mod results;
mod memory;

use std::ptr::{null, null_mut};
use std::marker::PhantomData;
use libc;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

use crate::connection::Connection;
use crate::types::{
    DescriptorsProvider,
    TypeDescriptor
};

use self::results::{
    ResultProcessor,
    QueryIterator
};
use self::params::ParamsProcessor;

pub use self::results::{
    ResultsProvider,
    ResultSet,
    ResultValue
};
pub use self::params::{
    ParamsProjection,
    ParamsProvider,
    ParamValue
};

use crate::OracleResult;

/// Generic prepared statement
pub struct Statement<'conn> {
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt
}

/// Statement with parameters (bindings)
pub struct BindedStatement<'conn, P> where P: ParamsProvider {
    stmt:    Statement<'conn>,
    params:  ParamsProcessor<'conn, P>,
    _params: std::marker::PhantomData<P>
}

/// Statement with ResultSet (defined result)
pub struct Query<'conn,R> where R: ResultsProvider {
    stmt:    Statement<'conn>,
    results: ResultProcessor<'conn, R>,
    _result: std::marker::PhantomData<R>
}

/// Statement with ResultSet for only ONE row in result
pub struct QueryOne<'conn,R> where R: ResultsProvider {
    stmt:    Statement<'conn>,
    results: ResultProcessor<'conn, R>,
    _result: std::marker::PhantomData<R>
}

/// Statement with parameters and ResultSet
pub struct BindedQuery<'conn,R,P>
    where R: ResultsProvider,
          P: ParamsProvider
{
    stmt:    Statement<'conn>,
    results: ResultProcessor<'conn, R>,
    params:  ParamsProcessor<'conn, P>,
    _result: std::marker::PhantomData<R>,
    _params: std::marker::PhantomData<P>
}

/// Statement with parameters and ResultSet for only ONE row in result
pub struct BindedQueryOne<'conn,R,P>
    where R: ResultsProvider,
          P: ParamsProvider {
    stmt:    Statement<'conn>,
    results: ResultProcessor<'conn, R>,
    params:  ParamsProcessor<'conn, P>,
    _result: std::marker::PhantomData<R>,
    _params: std::marker::PhantomData<P>
}

impl <'conn> Statement<'conn> {
    pub(crate) fn new<'s>(conn: &'conn Connection, sql:  &'s str) -> OracleResult<Statement<'conn>> {
        let stmthp = oci::stmt_prepare(conn.svchp, conn.errhp, sql)?;
        Ok( Statement { conn, stmthp } )
    }

    /// Prepare oracle statement with default prefetch 10 rows
    pub fn query<R: ResultsProvider>(self) -> OracleResult<Query<'conn,R>> {
        Query::new(self, 10)
    }

    /// Prepare oracle statement with default prefetch 1 row
    pub fn query_one<R: ResultsProvider>(self) -> OracleResult<QueryOne<'conn,R>> {
        QueryOne::new(self)
    }

    /// Prepare oracle statement with custom prefetch rows
    pub fn query_many<R: ResultsProvider>(self, prefetch_rows: usize) -> OracleResult<Query<'conn,R>> {
        Query::new(self, prefetch_rows)
    }

    /// Bind parameters descriptions to statement
    pub fn params<P: ParamsProvider>(mut self) -> OracleResult<BindedStatement<'conn,P>> {
        BindedStatement::new(self)
    }

    /// Execute generic statement
    pub fn execute(&mut self) -> OracleResult<()> {
        oci::stmt_execute(self.conn.svchp, self.stmthp, self.conn.errhp, 0, 0)?;
        Ok(())
    }

}

impl Drop for Statement<'_> {
    fn drop(&mut self) {
        oci::stmt_release(self.stmthp, self.conn.errhp);
    }
}

impl <'conn,R> Query<'conn,R> where R: ResultsProvider {
    fn new(stmt: Statement<'conn>, prefetch_rows: usize) -> OracleResult<Query<'conn,R>> {
        let prefetch_rows = if prefetch_rows > 0 {
            prefetch_rows
        } else {
            10
        };
        let results = ResultProcessor::new(stmt.conn, stmt.stmthp, prefetch_rows)?;
        Ok( Query { stmt, results, _result: PhantomData })
    }

    #[inline]
    pub fn fetch_iter<'iter>(&'iter mut self) -> OracleResult<QueryIterator<'iter, 'conn, R>> {
        self.stmt.execute()?;
        self.results.fetch_iter()
    }

    #[inline]
    pub fn fetch_list(&mut self) -> OracleResult<Vec<R>> {
        self.stmt.execute()?;
        self.results.fetch_list()
    }

}

impl <'conn,R> QueryOne<'conn,R> where R: ResultsProvider {
    fn new(stmt: Statement<'conn>) -> OracleResult<QueryOne<'conn,R>> {
        let results = ResultProcessor::new(stmt.conn, stmt.stmthp, 1)?;
        Ok( QueryOne { stmt, results, _result: PhantomData })
    }

    #[inline]
    pub fn fetch(&mut self) -> OracleResult<R> {
        self.stmt.execute()?;
        self.results.fetch_one()
    }

}

impl <'conn,P> BindedStatement<'conn,P> where P: ParamsProvider {
    fn new<'s>(stmt: Statement<'s>) -> OracleResult<BindedStatement<'s, P>> {
        let params = ParamsProcessor::new(stmt.conn, stmt.stmthp)?;
        Ok( BindedStatement { stmt, params, _params: PhantomData } )
    }

    /// Prepare oracle statement with default 10 prefetch rows
    pub fn query<R: ResultsProvider>(self) -> OracleResult<BindedQuery<'conn,R, P>> {
        BindedQuery::new(self, 10)
    }

    /// Prepare oracle statement with default prefetch 1 row
    pub fn query_one<R: ResultsProvider>(self) -> OracleResult<BindedQueryOne<'conn,R, P>> {
        BindedQueryOne::new(self)
    }

    /// Prepare oracle statement with default prefetch 1 row
    pub fn query_many<R: ResultsProvider>(self, prefetch_rows: usize) -> OracleResult<BindedQuery<'conn,R, P>> {
        BindedQuery::new(self, prefetch_rows)
    }

    /// Execute generic statement with params
    pub fn execute(&mut self, params: P) -> Result<(),oci::OracleError> {
        params.project_values(&mut self.params.projection);
        self.stmt.execute()
    }

}

impl <'conn,R, P> BindedQuery<'conn,R, P> where R: ResultsProvider,
                                                P: ParamsProvider {
    fn new(binded_stmt: BindedStatement<'conn,P>, prefetch_rows: usize) -> OracleResult<BindedQuery<'conn, R, P>> {
        let stmt = binded_stmt.stmt;
        let params = binded_stmt.params;

        let prefetch_rows = if prefetch_rows > 0 {
            prefetch_rows
        } else {
            1
        };

        let results = ResultProcessor::new(stmt.conn, stmt.stmthp, prefetch_rows)?;
        Ok( BindedQuery { stmt, results, params, _result: PhantomData, _params: PhantomData } )
    }

    #[inline]
    pub fn fetch_iter<'iter>(&'iter mut self, params: &P) -> Result<QueryIterator<'iter, 'conn, R>, oci::OracleError> {
        params.project_values(&mut self.params.projection);
        self.stmt.execute()?;
        self.results.fetch_iter()
    }

    #[inline]
    pub fn fetch_list(&mut self, params: &P) -> Result<Vec<R>, oci::OracleError> {
        params.project_values(&mut self.params.projection);
        self.stmt.execute()?;
        self.results.fetch_list()
    }

}

impl <'conn,R, P> BindedQueryOne<'conn,R, P> where R: ResultsProvider,
                                                   P: ParamsProvider {
    fn new(binded_stmt: BindedStatement<'conn,P>) -> OracleResult<BindedQueryOne<'conn,R, P>> {
        let stmt = binded_stmt.stmt;
        let results = ResultProcessor::new(stmt.conn, stmt.stmthp, 1)?;
        let params = binded_stmt.params;
        Ok( BindedQueryOne { stmt, results, params, _result: PhantomData, _params: PhantomData })
    }

    #[inline]
    pub fn fetch(&mut self, params: P) -> OracleResult<R> {
        params.project_values(&mut self.params.projection);
        self.stmt.execute()?;
        self.results.fetch_one()
    }

}
