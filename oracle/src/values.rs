use std::mem::transmute;
use std::ptr;

use crate::statement::{ParamValue, ResultValue};
use crate::ValueProjector;
use crate::SqlType;

// integer types, must be used only for primitive types
// TODO: optional types (ValueProjector)
macro_rules! convert_sql_and_primitive {
    ($T:ty) => {

        impl From<ResultValue> for $T {
            fn from(v: ResultValue) -> $T {
                v.map_or(Default::default(),|valp,_|unsafe { transmute::<*const u8, &$T>(valp) }.to_owned())
            }
        }

        impl From<ResultValue> for Option<$T> {
            fn from(v: ResultValue) -> Option<$T> {
                v.map(|valp,_|unsafe { transmute::<*const u8, &$T>(valp) }.to_owned())
            }
        }

        impl ValueProjector<$T> for $T {
            fn project_value(&self, projection: &mut ParamValue) {
                projection.project(self, |data, _| {
                    unsafe {
                        *( transmute::<*mut u8, &mut $T>(data) ) = *self;
                        0
                    }
                });
            }
        }

    }
}

convert_sql_and_primitive!(i16);
convert_sql_and_primitive!(u16);

convert_sql_and_primitive!(i32);
convert_sql_and_primitive!(u32);

convert_sql_and_primitive!(i64);
convert_sql_and_primitive!(u64);

convert_sql_and_primitive!(f64);

// TODO: From for Varchar, ValueProjector for Varchar

// String type, in Oracle NULL String is Empty String

impl From<ResultValue> for String {
    fn from(v: ResultValue) -> String {
        v.map_or(String::new(),|valp,len| {
            let str_len = len as usize;
            let mut dst = Vec::with_capacity(str_len) as Vec<u8>;
            unsafe {
                dst.set_len(str_len);
                ptr::copy(valp, dst.as_mut_ptr(), str_len);
                String::from_utf8_unchecked(dst)
            }
        })
    }
}

impl ValueProjector<String> for String {
    fn project_value(&self, projection: &mut ParamValue) {
        projection.project(self, |data, indp| {
            let str_len = self.len();
            unsafe {
                if str_len == 0 {
                    *indp = -1;
                } else {
                    ptr::copy(self.as_ptr(), data, str_len);
                }
                str_len
            }
        });
    }
}

impl ValueProjector<&str> for &str {
    fn project_value(&self, projection: &mut ParamValue) {
        projection.project(self, | data, indp| {
            let str_len = self.len();
            unsafe {
                if str_len == 0 {
                    *indp = -1;
                } else {
                    ptr::copy(self.as_ptr(), data, str_len);
                }
                str_len
            }
        });
    }
}

// boolean type mapped to u16 (INT TYPE IN DB), NULL is False

impl From<ResultValue> for bool {
    fn from(v: ResultValue) -> bool {
        let int_val = v.map_or(0,|valp,_| unsafe { transmute::<*const u8, &u16>(valp) }.to_owned());
        int_val == 0
    }
}

impl ValueProjector<bool> for bool {
    fn project_value(&self, projection: &mut ParamValue) {
        projection.project(self, |data, _| {
            let val: u16 = if *self { 1 } else { 0 };
            unsafe {
                *( transmute::<*mut u8, &mut u16>(data) ) = val;
                0
            }
        });
    }
}

// Date and Datetime
use chrono::prelude::*;
use crate::types::{SqlDate, SqlDateTime};

// TODO: Datetime have 7 bytes
// TODO: Timestamp have 11 bytes

impl From<ResultValue> for SqlDate {
    fn from(v: ResultValue) -> SqlDate {
        v.map_or(Local::now().date(),date_from_row)
    }
}

impl From<ResultValue> for SqlDateTime {
    fn from(v: ResultValue) -> SqlDateTime {
        v.map_or(Local::now(),datetime_from_row)
    }
}

impl From<ResultValue> for Option<SqlDate> {
    fn from(v: ResultValue) -> Option<SqlDate> {
        v.map(date_from_row)
    }
}

impl From<ResultValue> for Option<SqlDateTime> {
    fn from(v: ResultValue) -> Option<SqlDateTime> {
        v.map(datetime_from_row)
    }
}

impl ValueProjector<SqlDate> for SqlDate {
    fn project_value(&self, projection: &mut ParamValue) {
        projection.project(self, |data, _| date_to_row(self, data));
    }
}


impl ValueProjector<SqlDate> for SqlDateTime {
    fn project_value(&self, projection: &mut ParamValue) {
        projection.project(self, |data, _| datetime_to_row(self, data));
    }
}


// TODO: optional converters (ValueProjector) for date and datetime

fn date_from_row(valp: *const u8, len: u16) -> Date<Local> {
    assert!(len == 7, "Oracle Date length must be 7 bypes");
    let vec = unsafe { transmute::<*const u8, &[u8; 7]>(valp) };

    let y = (vec[0] as i32 - 100)*100 + vec[1] as i32 - 100;
    let m = vec[2] as u32;
    let d = vec[3] as u32;

    Local.ymd(y,m,d)
}

fn date_to_row(source: &Date<Local>, data: *mut u8) -> usize {
    let century = (source.year() / 100 + 100) as u8;
    let year = (source.year() % 100 + 100) as u8;
    unsafe {
        *data = century;
        *data.offset(1) = year;
        *data.offset(2) = source.month() as u8;
        *data.offset(3) = source.day() as u8;
        *data.offset(4) = 1;  // hour
        *data.offset(5) = 1;  // minute
        *data.offset(6) = 1;  // second
        0
    }
}


fn datetime_from_row(valp: *const u8, len: u16) -> DateTime<Local> {
    assert!(len == 7, "Oracle Datetime length must be 7 bypes");
    // assert!(len == 11, "Oracle Date length must be 11 bypes");
    let vec = unsafe { transmute::<*const u8, &[u8; 11]>(valp) };

    let y = (vec[0] as i32 - 100) * 100 + vec[1] as i32 - 100;
    let m = vec[2] as u32;
    let d = vec[3] as u32;

    let hh = vec[4] as u32;
    let mm = vec[5] as u32;
    let ss = vec[6] as u32;

    Local.ymd(y, m, d).and_hms(hh - 1, mm - 1, ss - 1)
}

fn datetime_to_row(source: &DateTime<Local>, data: *mut u8) -> usize {
    let century = (source.year() / 100 + 100) as u8;
    let year = (source.year() % 100 + 100) as u8;
    unsafe {
        *data = century;
        *data.offset(1) = year;
        *data.offset(2) = source.month() as u8;
        *data.offset(3) = source.day() as u8;
        *data.offset(4) = source.hour() as u8 + 1;
        *data.offset(5) = source.minute() as u8 + 1;
        *data.offset(6) = source.second() as u8 + 1;
        0
    }
}

use std::convert::TryFrom;

impl ResultValue {
    pub fn try_to_string(self, tp: &SqlType) -> Result<String, &'static str> {
        let result = match tp {
            SqlType::Varchar => {
                let v: String = self.into();
                format!("\"{}\"",v)
            },
            SqlType::Int16 => {
                let v: i16 = self.into();
                v.to_string()
            },
            SqlType::Int32 => {
                let v: i32 = self.into();
                v.to_string()
            },
            SqlType::Int64 => {
                let v: i64 = self.into();
                v.to_string()
            },
            SqlType::Float64 => {
                let v: f64 = self.into();
                v.to_string()
            },
            SqlType::DateTime => {
                let v: SqlDateTime = self.into();
                format!("\"{}\"", v.to_rfc3339())
            }
            _ => return Err("\"not-implemented\"")
        };
        Ok(result)
    }
}


