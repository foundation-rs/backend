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

// SEE: https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html

/// String with fixed predefined length
pub struct Varchar<'a, const PREFETCH: usize> (
    Cow<'a, str>
);

impl <'a, const PREFETCH: usize> Varchar<'a, PREFETCH> {
    pub fn new_owned(s: String) -> Varchar<'a, PREFETCH> {
        Varchar(Cow::Owned(s))
    }
    pub fn new_borrowed(s: &'a str) -> Varchar<'a, PREFETCH> {
        Varchar(Cow::Borrowed(s))
    }
    pub fn as_ref(&self) -> &str {
        &self.0.as_ref()
    }
    pub fn into_owned(self) -> String {
        self.0.into_owned()
    }
}

use std::ops::Deref;

impl <'a, const PREFETCH: usize> Deref for Varchar<'a, PREFETCH> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0.as_ref()
    }
}