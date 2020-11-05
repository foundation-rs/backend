use chrono::prelude::*;

// Date and Datetime

pub type SqlDate = Date<Local>;
pub type SqlDateTime = DateTime<Local>;

// TODO: Datetime have 7 bytes
// TODO: Timestamp have 11 bytes

