use std::convert::TryInto;

// TODO: inconsistency between timestamp and datetime
use crate::{
    ParamsProjection,
    ParamsProvider,
    ResultsProvider,
    ResultSet,
    TypeDescriptor,
    Member,
    Identifier,
    ValueProjector,
    SQLParams,
    SQLResults
};
use crate::types::*;
use crate::statement::ResultValue;

// impl metainfo for singular primitive types

/// general params/result provider
pub struct GeneralMetaProvider<T> {
    _marker: std::marker::PhantomData<T>
}

impl <T> GeneralMetaProvider<T> {
    pub fn new() -> GeneralMetaProvider<T> {
        GeneralMetaProvider { _marker: std::marker::PhantomData }
    }
}

impl <T: 'static> SQLResults for T where T: TypeDescriptorProducer<T> + From<ResultValue> {
    fn provider() -> Box<dyn ResultsProvider<Self>> {
        Box::new(GeneralMetaProvider::new())
    }
}

impl <T> ResultsProvider<T> for GeneralMetaProvider<T>
    where T: TypeDescriptorProducer<T> + From<ResultValue> {
    fn sql_descriptors(&self) -> Vec<TypeDescriptor> {
        vec![T::produce()]
    }
    fn gen_result(&self, rs: ResultSet) -> T {
        let values: [ResultValue; 1] = rs.try_into().unwrap();
        values[0].into()
    }
}

// implement params provider (top-level trait for compile-time params) for singular type
impl <T> SQLParams for T
    where T: 'static + TypeDescriptorProducer<T> + ValueProjector<T> {
    fn provider() -> Box<dyn ParamsProvider<Self>> {
        Box::new(GeneralMetaProvider::new())
    }
}

// implement params provider for singular type
impl <T> ParamsProvider<T> for GeneralMetaProvider<T>
    where T: 'static + TypeDescriptorProducer<T> + ValueProjector<T> {
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
        Box::new(GeneralPairProvider::new())
    }
}

struct GeneralPairProvider<T,V> {
    _m0: std::marker::PhantomData<T>,
    _m1: std::marker::PhantomData<V>,
}

impl <T,V> GeneralPairProvider<T,V> {
    pub fn new() -> GeneralPairProvider<T,V> {
        GeneralPairProvider {
            _m0: std::marker::PhantomData,
            _m1: std::marker::PhantomData
        }
    }
}

// implement params provider for pair tuple
impl <T,V> ParamsProvider<(T,V)> for GeneralPairProvider<T,V>
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

// implement params provider for pair tuple
impl <T,V> ResultsProvider<(T,V)> for GeneralPairProvider<T,V>
    where T: TypeDescriptorProducer<T> + From<ResultValue>,
          V: TypeDescriptorProducer<V> + From<ResultValue>,
{
    fn sql_descriptors(&self) -> Vec<TypeDescriptor> {
        vec![T::produce(), V::produce()]
    }
    fn gen_result(&self, rs: ResultSet) -> (T,V) {
        let values: [ResultValue; 2] = rs.try_into().unwrap();
        (values[0].into(),values[1].into())
    }
}
