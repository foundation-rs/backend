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
pub struct Query<'conn,P,R> where R: ResultsProvider {
    stmt:    Statement<'conn, P>,
    prefetch_rows: usize,
    results: Box<ResultProcessor<'conn, R>>
}

pub struct QueryIterator<'iter, 'conn: 'iter, P, R: 'conn> where R: ResultsProvider {
    stmt:    Statement<'conn, P>,
    results:  Box<ResultProcessor<'conn, R>>,
    iterator_ptr: *mut ResultIterator<'iter,'conn, R>
}

impl <'conn,P,R: 'conn> Query<'conn,P,R> where R: ResultsProvider {
    pub(crate) fn new(stmt: Statement<'conn,P>, prefetch_rows: usize) -> OracleResult<Query<'conn,P,R>> {
        let results = Box::new( ResultProcessor::new(stmt.conn, stmt.stmthp, prefetch_rows)? );
        Ok( Query { stmt, prefetch_rows, results })
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
                Ok(v) => result.push(v),
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
            Some(v) => v,
            None => Err(oci::OracleError::new("The request returned no data".to_owned(), "statement.fetch_one"))
        }
    }

}

impl <'iter, 'conn: 'iter, P, R: 'conn> QueryIterator<'iter,'conn, P, R> where R: ResultsProvider {
    fn new(query: Query<'conn,P,R>) -> OracleResult<QueryIterator<'iter,'conn, P, R>> {
        let stmt= query.stmt;
        let results= query.results;

        // transmute boxed ResultIterator into raw pointer
        // because Rust have problems with self-referencials structs
        let iterator_ptr = {
            let iterator = Box::new(results.fetch_iter()? );
            // by transmute rust don't auto-drop raw pointer and forget to drop iterator
            unsafe { core::mem::transmute(iterator) }
        };

        Ok( QueryIterator { stmt, results, iterator_ptr } )
    }

}

impl <'conn, 'iter: 'conn, P, R: 'conn> Iterator for QueryIterator<'conn, 'iter, P, R> where R: ResultsProvider {
    type Item = oci::OracleResult<R>;

    fn next(&mut self) -> Option<oci::OracleResult<R>> {
        unsafe {
            (*self.iterator_ptr).next()
        }
    }
}

impl <'conn, 'iter: 'conn, P, R: 'conn> Drop for QueryIterator<'conn, 'iter, P, R> where R: ResultsProvider {
    fn drop(&mut self) {
        // manually drop ResultIterator with transmute it from raw pointer to original boxed value
        let _boxed: Box<ResultIterator<'_, '_, R>> = unsafe { core::mem::transmute(self.iterator_ptr) };
    }
}
