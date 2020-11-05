// TODO: inconsistency between timestamp and datetime
use crate::dates::*;
use crate::{
    ResultsProvider,
    ResultSet,
    TypeDescriptor
};
use crate::types::*;

// impl metainfo for singular primitive types

macro_rules! impl_results_provider {
    ($T:ty, $name:ident) => {

        impl ResultsProvider for $T {

            fn sql_descriptors() -> Vec<TypeDescriptor> {
                vec![$name]
            }

            fn from_resultset(rs: &ResultSet) -> Self {
                let s0 = &(rs[0]);
                s0.into()
            }
        }

    }
}

impl_results_provider!(u32, U32_SQLTYPE);
impl_results_provider!(i32, I32_SQLTYPE);
impl_results_provider!(bool, BOOL_SQLTYPE);

impl_results_provider!(SqlDate, DATE_SQLTYPE);
impl_results_provider!(SqlDateTime, TIMESTAMP_SQLTYPE);


impl ResultsProvider for String {
    fn from_resultset(rs: &ResultSet) -> Self {
        let s0 = &(rs[0]);
        s0.into()
    }
    fn sql_descriptors() -> Vec<TypeDescriptor> {
        vec![string_sqltype(128)]
    }
}
