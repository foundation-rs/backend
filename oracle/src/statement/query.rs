use std::marker::PhantomData;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

use crate::OracleResult;

use super::params::ParamsProvider;

use super::Statement;
use super::results::{
    ResultProcessor,
    ResultsProvider,
    ResultIterator
};

/// Statement with ResultSet (defined result)
pub struct Query<'conn,P,R, const PREFETCH: usize> where P: ParamsProvider, R: ResultsProvider {
    stmt:    Statement<'conn, P>,

    results: Box<ResultProcessor<'conn, R>>,
    _result: std::marker::PhantomData<R>
}

pub struct QueryIterator<'iter, 'conn: 'iter, P, R, const PREFETCH: usize> where P: ParamsProvider, R: ResultsProvider {
    query:        Option<Query<'conn,P,R, PREFETCH>>,
    iterator_ptr: *mut ResultIterator<'iter,'conn, R>
}

impl <'conn,P,R: 'conn,const PREFETCH: usize> Query<'conn,P,R,PREFETCH> where P: ParamsProvider, R: ResultsProvider {
    pub(crate) fn new(stmt: Statement<'conn,P>) -> OracleResult<Query<'conn,P,R,PREFETCH>> {
        let results = box ResultProcessor::new(stmt.conn, stmt.stmthp, PREFETCH)?;
        Ok( Query { stmt, results, _result: PhantomData })
    }

    pub fn fetch_iter<'iter>(self, params: P) -> OracleResult<QueryIterator<'iter, 'conn, P, R, PREFETCH>> {
        assert!(PREFETCH > 1);
        self.stmt.set_params(params)?;
        
        // transmute boxed ResultIterator into raw pointer
        // because Rust have problems with self-referencials structs
        let iterator_ptr = {
            let iterator = box self.results.fetch_iter()?;
            // println!("fetch_iter::ResultIterator::boxed");
            // by transmute rust don't auto-drop raw pointer and forget to drop iterator
            unsafe { core::mem::transmute(iterator) }
        };

        // println!("fetch_iter::ResultIterator::before ok");

        Ok(QueryIterator::new(self, iterator_ptr))
    }

    #[inline]
    pub fn fetch_list(&self, params: P) -> OracleResult<Vec<R>> {
        assert!(PREFETCH > 1);
        let mut result = Vec::with_capacity(PREFETCH);

        self.stmt.set_params(params)?;
        let iterator = self.results.fetch_iter()?;

        for v in iterator {
            match v {
                Ok(v) => result.push(v),
                Err(err) => return Err(err)
            };
        }

        Ok( result )
    }

    #[inline]
    pub fn fetch_one(&self, params: P) -> OracleResult<R> {
        assert_eq!(PREFETCH, 1);

        self.stmt.set_params(params)?;
        let mut iterator = self.results.fetch_iter()?;

        match iterator.next() {
            Some(v) => v,
            None => Err(oci::OracleError::new("The request returned no data".to_owned(), "statement.fetch_one"))
        }
    }

}

impl <'iter, 'conn: 'iter, P, R, const PREFETCH: usize> QueryIterator<'iter,'conn, P, R, PREFETCH> where P: ParamsProvider, R: ResultsProvider {
    fn new(query: Query<'conn,P,R,PREFETCH>, iterator_ptr: *mut ResultIterator<'iter,'conn, R>) -> QueryIterator<'iter,'conn, P, R, PREFETCH> {
        QueryIterator { query: Some(query), iterator_ptr }
    }

    // consume iterator and extract query
    // we must use Option because we implement Drop trait for manually drop inner iterator
    pub fn release(mut self) -> Query<'conn,P,R,PREFETCH> {
        self.query.take().unwrap()
    }
}

impl <'conn, 'iter: 'conn, P, R, const PREFETCH: usize> Iterator for QueryIterator<'conn, 'iter, P, R, PREFETCH> where P: ParamsProvider, R: ResultsProvider {
    type Item = oci::OracleResult<R>;

    fn next(&mut self) -> Option<oci::OracleResult<R>> {
        unsafe { 
            (*self.iterator_ptr).next()
        }
    }
}

impl <'conn, 'iter: 'conn, P, R, const PREFETCH: usize> Drop for QueryIterator<'conn, 'iter, P, R, PREFETCH> where P: ParamsProvider, R: ResultsProvider {
    fn drop(&mut self) {
        // manually drop ResultIterator with transmute it from raw pointer to original boxed value
        let _boxed: Box<ResultIterator<'_, '_, R>> = unsafe { core::mem::transmute(self.iterator_ptr) };
    }
}
