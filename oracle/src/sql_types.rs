use std::borrow::Cow;
use chrono::prelude::*;

// Date and Datetime

// TODO: Oracle Datetime have 7 bytes
// TODO: Oracle Timestamp have 11 bytes

// converts to Oracle Datetime (4 bytes)
pub type SqlDate = Date<Local>;

// converts to Oracle Datetime (7 bytes)
pub type SqlDateTime = DateTime<Local>;

// converts to Oracle Timestamp (11 bytes)
// pub type SqlTimestamp = DateTime<Local>;

// TODO: construct Varchar from String or &str
// TODO: deref Varchar into String or &str

// TODO: type descriptor for Varchar with real length
// TODO: resultset and params for Varchar

// TODO: test working of Varchar with Query and Params macros

/// String with fixed predefined length
pub struct Varchar<'a, const PREFETCH: usize> (
    Cow<'a, str>
);
