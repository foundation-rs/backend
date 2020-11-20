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

