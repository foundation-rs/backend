mod error;
mod bindings;
mod constants;
mod functions;

pub use std::ffi::CString;

pub use bindings::{
    OCIError,
    OCIEnv,
    OCIServer,
    OCISession,
    OCISvcCtx,
    OCIStmt,
    c_void
};

pub use constants::{
    OCI_HTYPE_ERROR,
    OCI_HTYPE_ENV,
    OCI_HTYPE_SERVER,
    OCI_HTYPE_SVCCTX,
    OCI_HTYPE_SESSION,
    OCI_HTYPE_STMT,
    OCI_ATTR_SERVER,
    OCI_ATTR_SESSION,
    OCI_ATTR_ROWS_FETCHED,
    OCI_FETCH_NEXT
};

pub use error::{OracleError, OracleResult};

pub use functions::{
    env_create,
    terminate,
    handle_alloc,
    handle_free,
    server_attach,
    server_detach,
    attr_set,
    attr_get,
    prepare_auth,
    session_begin,
    session_end,
    commit,
    rollback,
    stmt_prepare,
    stmt_release,
    stmt_execute,
    stmt_fetch,
    define_by_pos,
    bind_by_pos
};
