mod memory;
pub mod params;
mod results;
mod query;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

use crate::connection::Connection;

use self::params::ParamsProcessor;

pub use self::results::{
    ResultsProvider,
    ResultSet,
    ResultValue,
    SQLResults
};
pub use self::params::{
    ParamsProjection,
    ParamsProvider,
    ParamValue,
    SQLParams
};
pub use self::query::{
    Query,
    QueryIterator
};

use crate::{OracleResult, OracleError};

/// Generic prepared statement with parameters (bindings)
/// Parameters may be () - Unit
pub struct Statement<'conn, P> {
    conn:     &'conn Connection,
    stmthp:   *mut oci::OCIStmt,
    provider: Box<dyn ParamsProvider<P>>,
    params:   ParamsProcessor
}

impl <'conn,P> Statement<'conn,P> {
    pub(crate) fn new<'s>(conn: &'conn Connection, sql:  &'s str, provider: Box<dyn ParamsProvider<P>>) -> OracleResult<Statement<'conn,P>> {
        let stmthp = oci::stmt_prepare(conn.svchp, conn.errhp, sql)?;
        let params = ParamsProcessor::new(conn, stmthp, provider.as_ref())?;
        Ok( Statement { conn, stmthp, provider, params } )
    }

    /// Prepare oracle statement with prefetch rows == 10
    pub fn query<R: 'conn + SQLResults>(self) -> OracleResult<Query<'conn,P,R>> {
        let provider = R::provider();
        Query::new(self, provider, 10)
    }

    /// Prepare oracle statement with prefetch rows == 1
    pub fn query_one<R: 'conn + SQLResults>(self) -> OracleResult<Query<'conn,P,R>> {
        let provider = R::provider();
        Query::new(self, provider, 1)
    }

    /// Prepare oracle statement with custom prefetch rows
    pub fn query_many<R: 'conn + SQLResults>(self, prefetch_rows: usize) -> OracleResult<Query<'conn,P,R>> {
        let provider = R::provider();
        Query::new(self, provider, prefetch_rows)
    }

    /// Prepare oracle statement with custom prefetch rows
    pub fn query_dynamic<'p, R: 'conn>(self, provider: Box<dyn ResultsProvider<R>>, prefetch_rows: usize) -> OracleResult<Query<'conn,P,R>> {
        Query::new(self, provider, prefetch_rows)
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
        self.provider.project_values(&params, projection.as_mut());
        Ok(())
    }

}

impl <P> Drop for Statement<'_,P> {
    fn drop(&mut self) {
        oci::stmt_release(self.stmthp, self.conn.errhp);
    }
}
