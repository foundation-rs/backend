use std::mem::size_of;

// Oracle Types, must be u16
#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
mod constants {
    pub const SQLT_CHR: u16 = 1;
    pub const SQLT_NUM: u16 = 2;
    pub const SQLT_INT: u16 = 3;
    pub const SQLT_FLT: u16 = 4;
    pub const SQLT_STR: u16 = 5;
    pub const SQLT_VNU: u16 = 6;
    pub const SQLT_PDN: u16 = 7;
    pub const SQLT_LNG: u16 = 8;
    pub const SQLT_VCS: u16 = 9;
    pub const SQLT_NON: u16 = 10;
    pub const SQLT_RID: u16 = 11;
    pub const SQLT_DAT: u16 = 12;
    pub const SQLT_VBI: u16 = 15;
    pub const SQLT_BFLOAT: u16 = 21;
    pub const SQLT_BDOUBLE: u16 = 22;
    pub const SQLT_BIN: u16 = 23;
    pub const SQLT_LBI: u16 = 24;
    pub const SQLT_UIN: u32 = 68;
    pub const SQLT_SLS: u16 = 91;
    pub const SQLT_LVC: u16 = 94;
    pub const SQLT_LVB: u16 = 95;
    pub const SQLT_AFC: u16 = 96;
    pub const SQLT_AVC: u16 = 97;
    pub const SQLT_IBFLOAT: u16 = 100;
    pub const SQLT_IBDOUBLE: u16 = 101;
    pub const SQLT_CUR: u16 = 102;
    pub const SQLT_RDD: u16 = 104;
    pub const SQLT_LAB: u16 = 105;
    pub const SQLT_OSL: u16 = 106;
    pub const SQLT_NTY: u16 = 108;
    pub const SQLT_REF: u16 = 110;
    pub const SQLT_CLOB: u16 = 112;
    pub const SQLT_BLOB: u16 = 113;
    pub const SQLT_BFILEE: u16 = 114;
    pub const SQLT_CFILEE: u16 = 115;
    pub const SQLT_RSET: u16 = 116;
    pub const SQLT_NCO: u16 = 122;
    pub const SQLT_VST: u16 = 155;
    pub const SQLT_ODT: u16 = 156;
    pub const SQLT_DATE: u16 = 184;
    pub const SQLT_TIME: u16 = 185;
    pub const SQLT_TIME_TZ: u16 = 186;
    pub const SQLT_TIMESTAMP: u16 = 187;
    pub const SQLT_TIMESTAMP_TZ: u16 = 188;
    pub const SQLT_INTERVAL_YM: u16 = 189;
    pub const SQLT_INTERVAL_DS: u16 = 190;
    pub const SQLT_TIMESTAMP_LTZ: u16 = 232;
    pub const SQLT_PNTY: u16 = 241;
    pub const SQLT_REC: u16 = 250;
    pub const SQLT_TAB: u16 = 251;
    pub const SQLT_BOL: u16 = 252;
}

/// incapsulate Oracle SQL Types
#[derive(Debug)]
pub struct TypeDescriptor {
    pub(crate) dtype: u16,
    pub(crate) size:  usize
}

impl TypeDescriptor {
    const fn new_typed<T>(dtype: u16) -> TypeDescriptor {
        TypeDescriptor { dtype, size: size_of::<T>()}
    }

    const fn new(dtype: u16, size: usize) -> TypeDescriptor {
        TypeDescriptor { dtype, size }
    }
}

macro_rules! type_desc {
    ($T:ty, $name:ident, $ora:ident) => {
        pub const $name: TypeDescriptor = TypeDescriptor::new_typed::<$T>(constants::$ora );
    }
}

// Interger types, signed and unsigned
type_desc!(i16, I16_SQLTYPE, SQLT_INT);
type_desc!(i32, I32_SQLTYPE, SQLT_INT);
type_desc!(i64, I64_SQLTYPE, SQLT_INT);

type_desc!(u16, U16_SQLTYPE, SQLT_INT);
type_desc!(u32, U32_SQLTYPE, SQLT_INT);
type_desc!(u64, U64_SQLTYPE, SQLT_INT);

// Float type
type_desc!(f64, F64_SQLTYPE, SQLT_FLT);

// Boolean type
type_desc!(bool, BOOL_SQLTYPE, SQLT_INT);

// Date and Datetime type
pub const DATE_SQLTYPE: TypeDescriptor = TypeDescriptor::new(constants::SQLT_DAT, 7 );
pub const DATETIME_SQLTYPE: TypeDescriptor = TypeDescriptor::new(constants::SQLT_DAT, 7 );
// pub const TIMESTAMP_SQLTYPE: TypeDescriptor = TypeDescriptor::new(constants::SQLT_TIMESTAMP, 11 );

pub trait TypeDescriptorProducer<T> {
    fn produce() -> TypeDescriptor {
        Self::produce_sized(0)
    }
    fn produce_sized(capacity: usize) -> TypeDescriptor;
}

// auto implement TypeDescriptorProducer for scalar types
macro_rules! impl_descriptors_producer {
    ($T:ty, $name:ident) => {

        impl TypeDescriptorProducer<$T> for $T {
            fn produce_sized(_capacity: usize) -> TypeDescriptor {
                $name
            }
        }

    }
}

impl_descriptors_producer!(i16, I16_SQLTYPE);
impl_descriptors_producer!(i32, I32_SQLTYPE);
impl_descriptors_producer!(i64, I64_SQLTYPE);

impl_descriptors_producer!(u16, U16_SQLTYPE);
impl_descriptors_producer!(u32, U32_SQLTYPE);
impl_descriptors_producer!(u64, U64_SQLTYPE);

// Float type
impl_descriptors_producer!(f64, F64_SQLTYPE);

// Boolean type
impl_descriptors_producer!(bool, BOOL_SQLTYPE);


// all about String type

pub fn string_sqltype(capacity: usize) -> TypeDescriptor {
    TypeDescriptor::new(constants::SQLT_CHR, capacity + 2)
}

impl TypeDescriptorProducer<String> for String {
    fn produce() -> TypeDescriptor {
        Self::produce_sized(128)
    }
    fn produce_sized(capacity: usize) -> TypeDescriptor {
        string_sqltype(capacity)
    }
}

impl TypeDescriptorProducer<&str> for &str {
    fn produce() -> TypeDescriptor {
        Self::produce_sized(128)
    }
    fn produce_sized(capacity: usize) -> TypeDescriptor {
        string_sqltype(capacity)
    }
}

// all about dates
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

// TODO: inconsistency between timestamp and datetime

impl_descriptors_producer!(SqlDate, DATE_SQLTYPE);
impl_descriptors_producer!(SqlDateTime, DATE_SQLTYPE);
// impl_descriptors_producer!(SqlTimestamp, TIMESTAMP_SQLTYPE);
