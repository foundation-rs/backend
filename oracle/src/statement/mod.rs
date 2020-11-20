mod memory;
pub mod params;
mod results;
mod query;

use std::marker::PhantomData;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

use crate::connection::Connection;

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
pub use self::query::{
    Query,
    QueryIterator
};

use crate::{OracleResult, OracleError};

/// Generic prepared statement with parameters (bindings)
/// Parameters may be () - Unit
pub struct Statement<'conn, P> where P: ParamsProvider {
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt,

    params:  ParamsProcessor<P>,
    _params: std::marker::PhantomData<P>
}

impl <'conn,P> Statement<'conn,P> where P: ParamsProvider {
    pub(crate) fn new<'s>(conn: &'conn Connection, sql:  &'s str) -> OracleResult<Statement<'conn,P>> {
        let stmthp = oci::stmt_prepare(conn.svchp, conn.errhp, sql)?;
        let params = ParamsProcessor::new(conn, stmthp)?;
        Ok( Statement { conn, stmthp, params, _params: PhantomData } )
    }

    /// Prepare oracle statement with prefetch rows == 10
    pub fn query<R: 'conn +  ResultsProvider>(self) -> OracleResult<Query<'conn,P,R>> {
        Query::new(self, 10)
    }

    /// Prepare oracle statement with prefetch rows == 1
    pub fn query_one<R: 'conn +  ResultsProvider>(self) -> OracleResult<Query<'conn,P,R>> {
        Query::new(self, 1)
    }

    /// Prepare oracle statement with custom prefetch rows
    pub fn query_many<R: 'conn +  ResultsProvider>(self, prefetch_rows: usize) -> OracleResult<Query<'conn,P,R>> {
        Query::new(self, prefetch_rows)
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
