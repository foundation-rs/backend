use std::mem::transmute;
use std::ptr;
use libc;

use crate::statement::{ParamValue, ResultValue};
use crate::ValueProjector;

// integer types, must be used only for primitive types

macro_rules! convert_sql_and_primitive {
    ($T:ty) => {

        impl From<&ResultValue> for $T {
            fn from(v: &ResultValue) -> $T {
                v.map_or(Default::default(),|valp,_|unsafe { transmute::<*const u8, &$T>(valp) }.to_owned())
            }
        }

        impl From<&ResultValue> for Option<$T> {
            fn from(v: &ResultValue) -> Option<$T> {
                v.map(|valp,_|unsafe { transmute::<*const u8, &$T>(valp) }.to_owned())
            }
        }

        impl ValueProjector<$T> for $T {
            fn project_value(&self, projection: &mut ParamValue) {
                projection.project(self, |val, data, _| {
                    unsafe {
                        *( transmute::<*mut u8, &mut $T>(data) ) = *val;
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


// String type, in Oracle NULL String is Empty String

impl From<&ResultValue> for String {
    fn from(v: &ResultValue) -> String {
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
        projection.project(self, |val, data, indp| {
            let str_len = val.len();
            unsafe {
                if str_len == 0 {
                    *indp = -1;
                } else {
                    ptr::copy(val.as_ptr(), data, str_len);
                }
                str_len
            }
        });
    }
}

impl ValueProjector<&str> for &str {
    fn project_value(&self, projection: &mut ParamValue) {
        projection.project(self, |val, data, indp| {
            let str_len = val.len();
            unsafe {
                if str_len == 0 {
                    *indp = -1;
                } else {
                    ptr::copy(val.as_ptr(), data, str_len);
                }
                str_len
            }
        });
    }
}

// boolean type mapped to u16 (INT TYPE IN DB), NULL is False

impl From<&ResultValue> for bool {
    fn from(v: &ResultValue) -> bool {
        let int_val = v.map_or(0,|valp,_| unsafe { transmute::<*const u8, &u16>(valp) }.to_owned());
        int_val == 0
    }
}

impl ValueProjector<bool> for bool {
    fn project_value(&self, projection: &mut ParamValue) {
        projection.project(self, |val, data, _| {
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
use crate::dates::*;

// TODO: Datetime have 7 bytes
// TODO: Timestamp have 11 bytes

impl From<&ResultValue> for SqlDate {
    fn from(v: &ResultValue) -> SqlDate {
        v.map_or(Local::now().date(),|valp,len| {
            assert!(len == 7, "Oracle Date length must be 7 bypes");
            let vec = unsafe { transmute::<*const u8, &[u8; 7]>(valp) };

            let y = (vec[0] as i32 - 100)*100 + vec[1] as i32 - 100;
            let m = vec[2] as u32;
            let d = vec[3] as u32;

            Local.ymd(y,m,d)
        })
    }
}

impl From<&ResultValue> for SqlDateTime {
    fn from(v: &ResultValue) -> SqlDateTime {
        v.map_or(Local::now(),|valp,len| {
            assert!(len == 11, "Oracle Date length must be 11 bypes");
            let vec = unsafe { transmute::<*const u8, &[u8; 11]>(valp) };

            let y = (vec[0] as i32 - 100)*100 + vec[1] as i32 - 100;
            let m = vec[2] as u32;
            let d = vec[3] as u32;

            let hh = vec[4] as u32;
            let mm = vec[5] as u32;
            let ss = vec[6] as u32;

            Local.ymd(y,m,d).and_hms(hh,mm,ss)
        })
    }
}

// TODO: optional converters for date and datetime
