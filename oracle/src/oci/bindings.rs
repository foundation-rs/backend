/*
 * OCI STRUCTURES AND FUNCTIONS
 */

pub use ::std::os::raw::{c_void, c_long, c_ulong, c_uint, c_int, c_ushort, c_uchar};

// oracle error handle
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OCIError {
    _unused: [u8; 0],
}

// oracle environment handle
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OCIEnv {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OCIServer {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OCISession {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OCISvcCtx {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OCIStmt {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OCIBind {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OCIDefine {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OCISnapshot {
    _unused: [u8; 0],
}

// OCI functions

extern "C" {
    pub fn OCIErrorGet(
        hndlp: *mut c_void,
        recordno: c_uint,
        sqlstate: *mut c_uchar,
        errcodep: *mut c_int,
        bufp: *mut c_uchar,
        bufsiz: c_uint,
        type_: c_uint,
    ) -> c_int;
}

extern "C" {
    pub fn OCIEnvCreate(
        envp: *mut *mut OCIEnv,
        mode: c_uint,
        ctxp: *mut c_void,
        malocfp: Option<
            unsafe extern "C" fn(
                ctxp: *mut c_void,
                size: c_ulong,
            ) -> *mut c_void,
        >,
        ralocfp: Option<
            unsafe extern "C" fn(
                ctxp: *mut c_void,
                memptr: *mut c_void,
                newsize: c_ulong,
            ) -> *mut c_void,
        >,
        mfreefp: Option<
            unsafe extern "C" fn(
                ctxp: *mut c_void,
                memptr: *mut c_void,
            ),
        >,
        xtramem_sz: c_ulong,
        usrmempp: *mut *mut c_void,
    ) -> c_int;
}

extern "C" {
    pub fn OCITerminate(mode: c_uint) -> c_int;
}

extern "C" {
    pub fn OCIHandleAlloc(
        parenth: *const c_void,
        hndlpp: *mut *mut c_void,
        type_: c_uint,
        xtramem_sz: c_ulong,
        usrmempp: *mut *mut c_void,
    ) -> c_int;
}
extern "C" {
    pub fn OCIHandleFree(hndlp: *mut c_void, type_: c_uint) -> c_int;
}

extern "C" {
    pub fn OCIAttrGet(
        trgthndlp: *const c_void,
        trghndltyp: c_uint,
        attributep: *mut c_void,
        sizep: *mut c_uint,
        attrtype: c_uint,
        errhp: *mut OCIError,
    ) -> c_int;
}

extern "C" {
    pub fn OCIAttrSet(
        trgthndlp: *mut c_void,
        trghndltyp: c_uint,
        attributep: *mut c_void,
        size: c_uint,
        attrtype: c_uint,
        errhp: *mut OCIError,
    ) -> c_int;
}

extern "C" {
    pub fn OCIServerAttach(
        srvhp: *mut OCIServer,
        errhp: *mut OCIError,
        dblink: *const c_uchar,
        dblink_len: c_int,
        mode: c_uint,
    ) -> c_int;
}
extern "C" {
    pub fn OCIServerDetach(srvhp: *mut OCIServer, errhp: *mut OCIError, mode: c_uint) -> c_int;
}
extern "C" {
    pub fn OCISessionBegin(
        svchp: *mut OCISvcCtx,
        errhp: *mut OCIError,
        usrhp: *mut OCISession,
        credt: c_uint,
        mode: c_uint,
    ) -> c_int;
}
extern "C" {
    pub fn OCISessionEnd(
        svchp: *mut OCISvcCtx,
        errhp: *mut OCIError,
        usrhp: *mut OCISession,
        mode: c_uint,
    ) -> c_int;
}

extern "C" {
    pub fn OCITransCommit(svchp: *mut OCISvcCtx, errhp: *mut OCIError, flags: c_uint) -> c_int;
}
extern "C" {
    pub fn OCITransRollback(svchp: *mut OCISvcCtx, errhp: *mut OCIError, flags: c_uint) -> c_int;
}
extern "C" {
    pub fn OCITransPrepare(svchp: *mut OCISvcCtx, errhp: *mut OCIError, flags: c_uint) -> c_int;
}

extern "C" {
    pub fn OCIStmtPrepare2(
        svchp: *mut OCISvcCtx,
        stmtp: *mut *mut OCIStmt,
        errhp: *mut OCIError,
        stmt: *const c_uchar,
        stmt_len: c_uint,
        key: *const c_uchar,
        key_len: c_uint,
        language: c_uint,
        mode: c_uint,
    ) -> c_int;
}
extern "C" {
    pub fn OCIStmtRelease(
        stmtp: *mut OCIStmt,
        errhp: *mut OCIError,
        key: *const c_uchar,
        key_len: c_uint,
        mode: c_uint,
    ) -> c_int;
}
extern "C" {
    pub fn OCIStmtExecute(
        svchp: *mut OCISvcCtx,
        stmtp: *mut OCIStmt,
        errhp: *mut OCIError,
        iters: c_uint,
        rowoff: c_uint,
        snap_in: *const OCISnapshot,
        snap_out: *mut OCISnapshot,
        mode: c_uint,
    ) -> c_int;
}

extern "C" {
    pub fn OCIDefineByPos(
        stmtp: *mut OCIStmt,
        defnp: *mut *mut OCIDefine,
        errhp: *mut OCIError,
        position: c_uint,
        valuep: *mut c_void,
        value_sz: c_int,
        dty: c_ushort,
        indp: *mut c_void,
        rlenp: *mut c_ushort,
        rcodep: *mut c_ushort,
        mode: c_uint,
    ) -> c_int;
}

extern "C" {
    pub fn OCIDefineByPos2(
        stmtp: *mut OCIStmt,
        defnp: *mut *mut OCIDefine,
        errhp: *mut OCIError,
        position: c_uint,
        valuep: *mut c_void,
        value_sz: c_long,
        dty: c_ushort,
        indp: *mut c_void,
        rlenp: *mut c_uint,
        rcodep: *mut c_ushort,
        mode: c_uint,
    ) -> c_int;
}

extern "C" {
    pub fn OCIBindByPos2(
        stmtp: *mut OCIStmt,
        bindp: *mut *mut OCIBind,
        errhp: *mut OCIError,
        position: c_uint,
        valuep: *mut c_void,
        value_sz: c_long,
        dty: c_ushort,
        indp: *mut c_void,
        alenp: *mut c_uint,
        rcodep: *mut c_ushort,
        maxarr_len: c_uint,
        curelep: *mut c_uint,
        mode: c_uint,
    ) -> c_int;
}
extern "C" {
    pub fn OCIBindByName2(
        stmtp: *mut OCIStmt,
        bindp: *mut *mut OCIBind,
        errhp: *mut OCIError,
        placeholder: *const c_uchar,
        placeh_len: c_int,
        valuep: *mut c_void,
        value_sz: c_long,
        dty: c_ushort,
        indp: *mut c_void,
        alenp: *mut c_uint,
        rcodep: *mut c_ushort,
        maxarr_len: c_uint,
        curelep: *mut c_uint,
        mode: c_uint,
    ) -> c_int;
}

extern "C" {
    pub fn OCIStmtFetch2(
        stmtp: *mut OCIStmt,
        errhp: *mut OCIError,
        nrows: c_uint,
        orientation: c_ushort,
        scrollOffset: c_int,
        mode: c_uint,
    ) -> c_int;
}
