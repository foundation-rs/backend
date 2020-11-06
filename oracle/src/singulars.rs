// TODO: inconsistency between timestamp and datetime
use crate::dates::*;
use crate::{
    ParamsProjection,
    ParamsProvider,
    ResultsProvider,
    ResultSet,
    TypeDescriptor,
    Member,
    Identifier,
    ValueProjector
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

macro_rules! impl_params_provider {
    ($T:ty, $name:ident) => {

        impl ParamsProvider for $T {
            fn members() -> Vec<Member> {
                vec![Member::new($name, Identifier::Unnamed)]
            }

            fn project_values(&self, projecton: &mut ParamsProjection) -> () {
                unsafe {
                    let p = projecton.get_unchecked_mut(0);
                    &self.project_value(p);
                }
            }
        }

    }
}

impl_results_provider!(u32, U32_SQLTYPE);
impl_results_provider!(i32, I32_SQLTYPE);
impl_results_provider!(bool, BOOL_SQLTYPE);

impl_results_provider!(SqlDate, DATE_SQLTYPE);
impl_results_provider!(SqlDateTime, TIMESTAMP_SQLTYPE);

impl_params_provider!(u32, U32_SQLTYPE);
impl_params_provider!(i32, I32_SQLTYPE);
impl_params_provider!(bool, BOOL_SQLTYPE);

/* TODO: implement project_value for dates
impl_params_provider!(SqlDate, DATE_SQLTYPE);
 */

impl ResultsProvider for String {
    fn from_resultset(rs: &ResultSet) -> Self {
        let s0 = &(rs[0]);
        s0.into()
    }
    fn sql_descriptors() -> Vec<TypeDescriptor> {
        vec![string_sqltype(128)]
    }
}

impl ParamsProvider for String {
    fn members() -> Vec<Member> {
        vec![Member::new(String::produce(), Identifier::Unnamed)]
    }

    fn project_values(&self, projecton: &mut ParamsProjection) -> () {
        unsafe {
            let p = projecton.get_unchecked_mut(0);
            &self.project_value(p);
        }
    }
}

