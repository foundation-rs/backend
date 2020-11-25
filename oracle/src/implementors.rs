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
    ValueProjector,
    SQLParams
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

// implement params provider (top-level trait for compile-time params) for singular type
impl <T> SQLParams for T
    where T: 'static + TypeDescriptorProducer<T> + ValueProjector<T> {
    fn provider() -> Box<dyn ParamsProvider<Self>> {
        Box::new(GeneralParamsProvider{ _marker: std::marker::PhantomData })
    }
}

/// general params provider for singular type
struct GeneralParamsProvider<T>
    where T: TypeDescriptorProducer<T> + ValueProjector<T> {
    _marker: std::marker::PhantomData<T>
}

// implement params provider for singular type
impl <T> ParamsProvider<T> for GeneralParamsProvider<T>
    where T: TypeDescriptorProducer<T> + ValueProjector<T> {
    fn members(&self) -> Vec<Member> {
        vec![
            Member::new(T::produce(), Identifier::Unnamed),
        ]
    }

    fn project_values(&self, params: &T, projecton: &mut ParamsProjection) {
        unsafe {
            let p = projecton.get_unchecked_mut(0);
            &params.project_value(p);
        }
    }
}

impl SQLParams for () {
    fn provider() -> Box<dyn ParamsProvider<Self>> {
        Box::new(())
    }
}

// implement params provider for singular type
impl ParamsProvider<()> for () {
    fn members(&self) -> Vec<Member> {
        vec![]
    }
    fn project_values(&self, _params: &(), _projecton: &mut ParamsProjection) {}
}

impl <T,V> SQLParams for (T,V)
    where T: 'static + TypeDescriptorProducer<T> + ValueProjector<T>,
          V: 'static + TypeDescriptorProducer<V> + ValueProjector<V> {
    fn provider() -> Box<dyn ParamsProvider<Self>> {
        Box::new(GeneralPairParamsProvider{
            _m0: std::marker::PhantomData,
            _m1: std::marker::PhantomData
        })
    }
}

struct GeneralPairParamsProvider<T,V>
    where T: TypeDescriptorProducer<T> + ValueProjector<T>,
          V: TypeDescriptorProducer<V> + ValueProjector<V> {
    _m0: std::marker::PhantomData<T>,
    _m1: std::marker::PhantomData<V>,
}

// implement params provider for pair tuple
impl <T,V> ParamsProvider<(T,V)> for GeneralPairParamsProvider<T,V>
    where T: TypeDescriptorProducer<T> + ValueProjector<T>,
          V: TypeDescriptorProducer<V> + ValueProjector<V>,
{
    fn members(&self) -> Vec<Member> {
        vec![
            Member::new(T::produce(), Identifier::Unnamed),
            Member::new(V::produce(), Identifier::Unnamed)
        ]
    }

    fn project_values(&self, params: &(T,V), projecton: &mut ParamsProjection) {
        unsafe {
            let p = projecton.get_unchecked_mut(0);
            &params.0.project_value(p);
        }
        unsafe {
            let p = projecton.get_unchecked_mut(1);
            &params.1.project_value(p);
        }
    }
}
