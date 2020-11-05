// Oracle driver version
pub const OCI_MAJOR_VERSION: u32 = 19;
pub const OCI_MINOR_VERSION: u32 = 3;

pub const OCI_DEFAULT: u32 = 0;
pub const OCI_THREADED: u32 = 1;

// credentials
pub const OCI_CRED_RDBMS: u32 = 1;

// handlers
pub const OCI_HTYPE_ENV: u32 = 1;
pub const OCI_HTYPE_ERROR: u32 = 2;
pub const OCI_HTYPE_SVCCTX: u32 = 3;
pub const OCI_HTYPE_SERVER: u32 = 8;
pub const OCI_HTYPE_SESSION: u32 = 9;
pub const OCI_HTYPE_STMT: u32 = 4;
pub const OCI_HTYPE_BIND: u32 = 5;
pub const OCI_HTYPE_DEFINE: u32 = 6;

// ERROR CODES
pub const OCI_SUCCESS: i32 = 0;
pub const OCI_SUCCESS_WITH_INFO: i32 = 1;
pub const OCI_NEED_DATA: i32 = 99;
pub const OCI_NO_DATA: i32 = 100;
pub const OCI_ERROR: i32 = -1;
pub const OCI_INVALID_HANDLE: i32 = -2;
pub const OCI_STILL_EXECUTING: i32 = -3123;
pub const OCI_CONTINUE: i32 = -24200;
pub const OCI_ROWCBK_DONE: i32 = -24201;

// other constants
pub const OCI_BATCH_MODE: u32 = 1;
pub const OCI_EXACT_FETCH: u32 = 2;
pub const OCI_STMT_SCROLLABLE_READONLY: u32 = 8;
pub const OCI_DESCRIBE_ONLY: u32 = 16;
pub const OCI_COMMIT_ON_SUCCESS: u32 = 32;
pub const OCI_NON_BLOCKING: u32 = 64;
pub const OCI_BATCH_ERRORS: u32 = 128;
pub const OCI_PARSE_ONLY: u32 = 256;
pub const OCI_NTV_SYNTAX: u32 = 1;

pub const OCI_PARAM_IN: u32 = 1;
pub const OCI_PARAM_OUT: u32 = 2;

// attributes
pub const OCI_ATTR_SERVER: u32 = 6;
pub const OCI_ATTR_SESSION: u32 = 7;
pub const OCI_ATTR_TRANS: u32 = 8;
pub const OCI_ATTR_USERNAME: u32 = 22;
pub const OCI_ATTR_PASSWORD: u32 = 23;
pub const OCI_ATTR_ROWS_FETCHED: u32 = 197;

// transactions
pub const OCI_TRANS_NEW: u32 = 1;
pub const OCI_TRANS_JOIN: u32 = 2;
pub const OCI_TRANS_RESUME: u32 = 4;
pub const OCI_TRANS_PROMOTE: u32 = 8;
pub const OCI_TRANS_READONLY: u32 = 256;
pub const OCI_TRANS_READWRITE: u32 = 512;
pub const OCI_TRANS_SERIALIZABLE: u32 = 1024;
pub const OCI_TRANS_TWOPHASE: u32 = 16777216;
pub const OCI_TRANS_WRITEBATCH: u32 = 1;
pub const OCI_TRANS_WRITENOWAIT: u32 = 8;

// Fetch direction, must be u16
pub const OCI_FETCH_CURRENT: u16 = 1;
pub const OCI_FETCH_NEXT: u16 = 2;
pub const OCI_FETCH_FIRST: u16 = 4;
pub const OCI_FETCH_LAST: u16 = 8;
pub const OCI_FETCH_PRIOR: u16 = 16;
pub const OCI_FETCH_ABSOLUTE: u16 = 32;
pub const OCI_FETCH_RELATIVE: u16 = 64;
