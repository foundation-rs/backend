#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

use crate::OracleResult;

use super::Statement;
use super::results::{
    ResultProcessor,
    ResultsProvider,
    ResultIterator
};

/// Statement with ResultSet (defined result)
pub struct Query<'conn,P,R> {
    stmt:    Statement<'conn, P>,
    prefetch_rows: usize,
    provider: Box<dyn ResultsProvider<R>>,
    results: Box<ResultProcessor<'conn>>
}

pub struct QueryIterator<'iter, 'conn: 'iter, P, R: 'conn> {
    stmt:    Statement<'conn, P>,
    provider: Box<dyn ResultsProvider<R>>,
    results:  Box<ResultProcessor<'conn>>,
    iterator_ptr: *mut ResultIterator<'iter,'conn>
}

impl <'conn,P,R: 'conn> Query<'conn,P,R> {
    pub(crate) fn new(stmt: Statement<'conn,P>, provider: Box<dyn ResultsProvider<R>>, prefetch_rows: usize) -> OracleResult<Query<'conn,P,R>> {
        let results = Box::new( ResultProcessor::new(stmt.conn, stmt.stmthp, provider.as_ref(), prefetch_rows)? );
        Ok( Query { stmt, prefetch_rows, provider, results })
    }

    pub fn fetch_iter<'iter>(self, params: P) -> OracleResult<QueryIterator<'iter, 'conn, P, R>> {
        assert!(self.prefetch_rows > 1);
        self.stmt.set_params(params)?;
        QueryIterator::new(self)
    }

    #[inline]
    pub fn fetch_list(&self, params: P) -> OracleResult<Vec<R>> {
        assert!(self.prefetch_rows > 1 && self.prefetch_rows <= 100);
        let mut result = Vec::with_capacity(self.prefetch_rows);

        self.stmt.set_params(params)?;
        let iterator = self.results.fetch_iter()?;

        for v in iterator {
            match v {
                Ok(v) => result.push(self.provider.gen_result(v)),
                Err(err) => return Err(err)
            };
        }

        Ok( result )
    }

    #[inline]
    pub fn fetch_one(&self, params: P) -> OracleResult<R> {
        assert_eq!(self.prefetch_rows, 1);

        self.stmt.set_params(params)?;
        let mut iterator = self.results.fetch_iter()?;

        match iterator.next() {
            Some(v) => v.map(|r|self.provider.gen_result(r)),
            None => Err(oci::OracleError::new("The request returned no data".to_owned(), "statement.fetch_one"))
        }
    }

}

impl <'iter, 'conn: 'iter, P, R: 'conn> QueryIterator<'iter,'conn, P, R> {
    fn new(query: Query<'conn,P,R>) -> OracleResult<QueryIterator<'iter,'conn, P, R>> {
        let stmt= query.stmt;
        let results= query.results;
        let provider = query.provider;

        // transmute boxed ResultIterator into raw pointer
        // because Rust have problems with self-referencials structs
        let iterator_ptr = {
            let iterator = Box::new(results.fetch_iter()? );
            // by transmute rust don't auto-drop raw pointer and forget to drop iterator
            unsafe { core::mem::transmute(iterator) }
        };

        Ok( QueryIterator { stmt, provider, results, iterator_ptr } )
    }

}

impl <'conn, 'iter: 'conn, P, R: 'conn> Iterator for QueryIterator<'conn, 'iter, P, R> {
    type Item = oci::OracleResult<R>;

    fn next(&mut self) -> Option<oci::OracleResult<R>> {
        unsafe {
            (*self.iterator_ptr)
                .next()
                .map(|r|r.map(|r|self.provider.gen_result(r)))
        }
    }
}

impl <'conn, 'iter, P, R> Drop for QueryIterator<'conn, 'iter, P, R> {
    fn drop(&mut self) {
        // manually drop ResultIterator with transmute it from raw pointer to original boxed value
        let _boxed: Box<ResultIterator<'_, '_>> = unsafe { core::mem::transmute(self.iterator_ptr) };
    }
}
