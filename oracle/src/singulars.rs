// TODO: inconsistency between timestamp and datetime
use crate::sql_types::*;
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

impl_results_provider!(u32, U32_SQLTYPE);
impl_results_provider!(i32, I32_SQLTYPE);
impl_results_provider!(bool, BOOL_SQLTYPE);

impl_results_provider!(SqlDate, DATE_SQLTYPE);
impl_results_provider!(SqlDateTime, DATETIME_SQLTYPE);


impl ResultsProvider for String {
    fn from_resultset(rs: &ResultSet) -> Self {
        let s0 = &(rs[0]);
        s0.into()
    }
    fn sql_descriptors() -> Vec<TypeDescriptor> {
        vec![string_sqltype(128)]
    }
}

// implement params provider for singular type
impl <T> ParamsProvider for T
    where T: TypeDescriptorProducer<T> + ValueProjector<T> {
    fn members() -> Vec<Member> {
        vec![
            Member::new(T::produce(), Identifier::Unnamed),
        ]
    }

    fn project_values(&self, projecton: &mut ParamsProjection) {
        unsafe {
            let p = projecton.get_unchecked_mut(0);
            &self.project_value(p);
        }
    }
}

// implement params provider for singular type
impl ParamsProvider for () {
    fn members() -> Vec<Member> {
        vec![]
    }
    fn project_values(&self, _projecton: &mut ParamsProjection) {}
}

// implement params provider for pair tuple
impl <T,V> ParamsProvider for (T,V)
    where T: TypeDescriptorProducer<T> + ValueProjector<T>,
          V: TypeDescriptorProducer<V> + ValueProjector<V> {
    fn members() -> Vec<Member> {
        vec![
            Member::new(T::produce(), Identifier::Unnamed),
            Member::new(V::produce(), Identifier::Unnamed)
        ]
    }

    fn project_values(&self, projecton: &mut ParamsProjection) {
        unsafe {
            let p = projecton.get_unchecked_mut(0);
            &self.0.project_value(p);
        }
        unsafe {
            let p = projecton.get_unchecked_mut(1);
            &self.1.project_value(p);
        }
    }
}
