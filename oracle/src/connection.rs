#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

use crate::environment::Environment;
use crate::{statement, OracleResult, ResultsProvider, ParamsProvider};

/// Connection to Oracle and server context
pub struct Connection {
    env: &'static Environment,
    srvhp: *mut oci::OCIServer,
    authp: *mut oci::OCISession,
    pub(crate) errhp: *mut oci::OCIError,
    pub(crate) svchp: *mut oci::OCISvcCtx,
}

/// connect to database
pub fn connect(db: &str, username: &str, passwd: &str) -> Result<Connection, oci::OracleError> {
    let env = Environment::get()?;
    let srvhp = oci::handle_alloc(env.envhp, oci::OCI_HTYPE_SERVER)? as *mut oci::OCIServer;
    let svchp = oci::handle_alloc(env.envhp, oci::OCI_HTYPE_SVCCTX)? as *mut oci::OCISvcCtx;

    let errhp = env.errhp;
    let res = oci::server_attach(srvhp, errhp, db);
    if let Err(err) = res {
        free_server_handlers(srvhp, svchp);
        return Err(err);
    };

    // set attribute server context in the service context
    oci::attr_set(svchp as *mut oci::c_void,
                  oci::OCI_HTYPE_SVCCTX,
                  srvhp as *mut oci::c_void,
                  0,
                  oci::OCI_ATTR_SERVER,
                  errhp)?;

    let authp = oci::prepare_auth(env.envhp, errhp, username, passwd)?;

    let res = oci::session_begin(svchp, errhp, authp);
    if let Err(err) = res {
        free_session_handler(authp);
        free_server_handlers(srvhp, svchp);
        return Err(err);
    };

    // set session context in the service context
    oci::attr_set(svchp as *mut oci::c_void, oci::OCI_HTYPE_SVCCTX,
                  authp as *mut oci::c_void, 0,
                  oci::OCI_ATTR_SESSION, errhp)?;


    return Ok( Connection::new(env, srvhp, authp, errhp, svchp ) );
}

impl Connection {
    fn new(env: &'static Environment,
           srvhp: *mut oci::OCIServer,
           authp: *mut oci::OCISession,
           errhp: *mut oci::OCIError,
           svchp: *mut oci::OCISvcCtx) -> Connection {
        Connection { env, srvhp, authp, errhp, svchp }
    }

    /// commit transaction with NO-WAIT option
    pub fn commit(&self) -> Result<(), oci::OracleError> {
        oci::commit(self.svchp, self.env.errhp)
    }

    /// rollback transation
    pub fn rollback(&self) -> Result<(), oci::OracleError> {
        oci::rollback(self.svchp, self.env.errhp)
    }

    /// Execute generic SQL statement
    pub fn execute<'conn,'s>(&'conn self, sql: &'s str) -> Result<(), oci::OracleError> {
        let mut st = statement::Statement::new(self, sql)?;
        st.execute()
    }

    /// Prepare generic oracle statement
    pub fn prepare<'conn,'s>(&'conn self, sql: &'s str) -> Result<statement::Statement<'conn>, oci::OracleError> {
        statement::Statement::new(self, sql)
    }

    /// Prepare query with default 10 prefetch rows
    pub fn query<R>(&self, sql: &str)
                    -> OracleResult<statement::Query<R>> where R: ResultsProvider {
        statement::Statement::new(self, sql)?.query()
    }

    /// Prepare query with 10 row
    pub fn query_one<R>(&self, sql: &str)
                    -> OracleResult<statement::QueryOne<R>> where R: ResultsProvider {
        statement::Statement::new(self, sql)?.query_one()
    }

    /// Prepare query with default 10 prefetch rows and params
    pub fn query_with_params<R,P>(&self, sql: &str)
                    -> OracleResult<statement::BindedQuery<R,P>>
        where R: ResultsProvider, P: ParamsProvider {
        statement::Statement::new(self, sql)?.params()?.query()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        oci::session_end(self.svchp, self.env.errhp, self.authp);
        oci::server_detach(self.srvhp, self.env.errhp);
        free_session_handler(self.authp);
        free_server_handlers(self.srvhp, self.svchp);
    }
}

fn free_session_handler(authp: *mut oci::OCISession) {
    if !authp.is_null() {
        oci::handle_free(authp as *mut oci::c_void, oci::OCI_HTYPE_SESSION);
    }
}

fn free_server_handlers(srvhp: *mut oci::OCIServer, svchp: *mut oci::OCISvcCtx) {
    if !svchp.is_null() {
        oci::handle_free(svchp as *mut oci::c_void, oci::OCI_HTYPE_SVCCTX);
    }
    if !srvhp.is_null() {
        oci::handle_free(srvhp as *mut oci::c_void, oci::OCI_HTYPE_SERVER);
    }
}
