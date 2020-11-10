pub mod params;
mod results;
mod memory;

use std::marker::PhantomData;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

use crate::connection::Connection;

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

use crate::{OracleResult, OracleError};

/// Generic prepared statement with parameters (bindings)
/// Parameters may be () - Unit
pub struct Statement<'conn, P> where P: ParamsProvider {
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt,

    params:  ParamsProcessor<'conn, P>,
    _params: std::marker::PhantomData<P>
}

/// Statement with ResultSet (defined result)
pub struct Query<'conn,P,R, const PREFETCH: usize> where P: ParamsProvider, R: ResultsProvider {
    stmt:    Statement<'conn, P>,

    results: ResultProcessor<'conn, R>,
    _result: std::marker::PhantomData<R>
}

impl <'conn,P> Statement<'conn,P> where P: ParamsProvider {
    pub(crate) fn new<'s>(conn: &'conn Connection, sql:  &'s str) -> OracleResult<Statement<'conn,P>> {
        let stmthp = oci::stmt_prepare(conn.svchp, conn.errhp, sql)?;
        let params = ParamsProcessor::new(conn, stmthp)?;
        Ok( Statement { conn, stmthp, params, _params: PhantomData } )
    }

    /// Prepare oracle statement with default prefetch 10 rows
    pub fn query<R: ResultsProvider>(self) -> OracleResult<Query<'conn,P, R, 10>> {
        Query::new(self)
    }

    /// Prepare oracle statement with default prefetch 1 row
    pub fn query_one<R: ResultsProvider>(self) -> OracleResult<Query<'conn,P,R,1>> {
        Query::new(self)
    }

    /// Prepare oracle statement with custom prefetch rows
    pub fn query_many<R: ResultsProvider, const PREFETCH: usize>(self) -> OracleResult<Query<'conn,P,R,PREFETCH>> {
        assert!(PREFETCH == 20 || PREFETCH == 50 || PREFETCH == 100 || PREFETCH == 200 || PREFETCH == 500 || PREFETCH == 1000 || PREFETCH == 5000 || PREFETCH == 10000);
        Query::new(self)
    }

    /// Execute generic statement with params
    pub fn execute(&self, params: P) -> OracleResult<()> {
        self.set_params(params)?;
        oci::stmt_execute(self.conn.svchp, self.stmthp, self.conn.errhp, 0, 0).map(|_| ())
    }

    pub(crate) fn set_params(&self, params: P) -> OracleResult<()> {
        let mut projection = self.params.projection
            .try_borrow_mut()
            .map_err(|err|OracleError::new(format!("Can not borrow params-projection for set-params: {}", err),"Statement::set_params"))?;
        params.project_values(projection.as_mut());
        Ok(())
    }

}

impl <P> Drop for Statement<'_,P> where P: ParamsProvider {
    fn drop(&mut self) {
        oci::stmt_release(self.stmthp, self.conn.errhp);
    }
}

impl <'conn,P,R,const PREFETCH: usize> Query<'conn,P,R,PREFETCH> where P: ParamsProvider, R: ResultsProvider {
    fn new(stmt: Statement<'conn,P>) -> OracleResult<Query<'conn,P,R,PREFETCH>> {
        let results = ResultProcessor::new(stmt.conn, stmt.stmthp, PREFETCH)?;
        Ok( Query { stmt, results, _result: PhantomData })
    }

    #[inline]
    pub fn fetch_iter<'iter>(&'iter self, params: P) -> OracleResult<QueryIterator<'iter, 'conn, R>> {
        assert!(PREFETCH > 1);
        self.stmt.set_params(params)?;
        self.results.fetch_iter()
    }

    #[inline]
    pub fn fetch_list(&self, params: P) -> OracleResult<Vec<R>> {
        assert!(PREFETCH > 1);
        self.stmt.set_params(params)?;
        self.results.fetch_list()
    }

    #[inline]
    pub fn fetch(&self, params: P) -> OracleResult<R> {
        assert_eq!(PREFETCH, 1);
        self.stmt.set_params(params)?;
        self.results.fetch_one()
    }

}
