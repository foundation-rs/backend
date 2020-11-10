use std::alloc::{alloc, dealloc, Layout};
use std::marker::PhantomData;
use libc;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

use crate::connection::Connection;
use crate::types::TypeDescriptor;
use crate::statement::memory::align_size_to;
use crate::OracleResult;

/// Contains row data for one item.
/// Used for result-set
pub enum ResultValue {
    Val {
        valp: *const u8,
        len: u16,
    },
    Nil
}

pub type ResultSet = Vec<ResultValue>;

pub trait ResultsProvider {
    fn sql_descriptors() -> Vec<TypeDescriptor>;
    fn from_resultset(rs: &ResultSet) -> Self;
}

impl <'a> ResultValue {

    /// Convert row data to concrete optional type
    #[inline]
    pub fn map<U,F>(&self, f: F)
                    -> Option<U> where F: FnOnce(*const u8, u16) -> U {
        match self {
            ResultValue::Val {valp, len} => Some(f(*valp, *len)),
            ResultValue::Nil => None
        }
    }

    /// Convert row data to concrete non-optional type
    #[inline]
    pub fn map_or<U,F: FnOnce(*const u8, u16) -> U>(&self, default: U, f: F) -> U {
        match self {
            ResultValue::Val {valp, len} => f(*valp, *len),
            ResultValue::Nil => default
        }
    }

}

pub(crate) struct ResultProcessor<'conn, R> where R: ResultsProvider {
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt,

    prefetch_rows: usize,
    sizes:         Vec<isize>,

    allocated_p:   *mut u8,    // pointer to a main allocated block
    allocated_layout: Layout,  // layout of allocated block

    values_p:      *const u8,  // pointer to values area
    indicators_p:  *const i16, // pointer to indicators area
    ret_lengths_p: *const u16, // pointer to return length area,

    _result: std::marker::PhantomData<R>
}

pub struct QueryIterator<'iter,'conn: 'iter, R> where R: ResultsProvider {
    processor: &'iter ResultProcessor<'conn, R>,
    done:      bool,
    initial_prefetched: u32,
    rows_fetched: u32,
    cursor_index: u32,
    _result: std::marker::PhantomData<R>
}

impl <'conn, R> ResultProcessor<'conn, R> where R: ResultsProvider {

    pub(crate) fn new(conn: &'conn Connection, stmthp: *mut oci::OCIStmt, prefetch_rows: usize)
                      -> Result<ResultProcessor<'conn, R>, oci::OracleError> {
        let descriptors = R::sql_descriptors();
        let columns_cnt = descriptors.len();
        let mut sizes = Vec::with_capacity(columns_cnt);

        let val_size =  descriptors.iter().map(|d| d.size ).sum::<usize>();

        // calc sized aligned for best allocation
        let area_size = align_size_to(val_size * prefetch_rows, 128);
        let inds_size = align_size_to(columns_cnt * prefetch_rows * 2, 64);

        let total_size = align_size_to(area_size + inds_size * 2, 256);

        let allocated_layout = Layout::array::<u8>(total_size).unwrap();
        let allocated_p = unsafe { alloc(allocated_layout) };

        if allocated_p.is_null() {
            panic!("failed to allocate memory for Result buffer");
        }

        let indicators_p = allocated_p as *const i16;
        let ret_lengths_p = unsafe { allocated_p.offset(inds_size as isize) } as *const u16;
        let values_p = unsafe { allocated_p.offset((inds_size *2) as isize) } as *const u8;

        let mut offset = 0;
        let mut offset_i = 0;

        unsafe {
            for (i,d) in descriptors.iter().enumerate() {
                let value_p = values_p.offset(offset) as *mut libc::c_void;
                let ind_p = indicators_p.offset(offset_i) as *mut libc::c_void;
                let rlen_p = ret_lengths_p.offset(offset_i) as *mut u16;

                offset += (d.size * prefetch_rows) as isize;
                offset_i += prefetch_rows as isize;

                oci::define_by_pos(stmthp, conn.errhp, (i + 1) as u32, value_p, ind_p, d.size as i32, rlen_p, d.dtype)?;

                sizes.push(d.size as isize);
            }
        }

        oci::set_prefetch_size(stmthp, conn.errhp, prefetch_rows as u32)?;

        Ok( ResultProcessor {conn, stmthp, prefetch_rows, sizes, allocated_p, allocated_layout, values_p, indicators_p, ret_lengths_p, _result: PhantomData} )
    }

    /// Execute generic statement with prefetch values
    fn execute_prefetch(&self, iters: u32) -> OracleResult<bool> {
        oci::stmt_execute(self.conn.svchp, self.stmthp, self.conn.errhp, iters, 0)
    }

    pub(crate) fn fetch_iter<'iter>(&'iter self) ->
    Result<QueryIterator<'iter, 'conn, R>, oci::OracleError> {
        let success = self.execute_prefetch(self.prefetch_rows as u32)?;
        let prefetched = self.initial_fetch(success)?;

        Ok(QueryIterator::new(self, prefetched))
    }

    pub(crate) fn fetch_list (&self)
                                  -> Result<Vec<R>, oci::OracleError> {
        let mut result = Vec::with_capacity(self.prefetch_rows);

        for v in self.fetch_iter()? {
            match v {
                Ok(v) => result.push(v),
                Err(err) => return Err(err)
            };
        }
        Ok( result )
    }

    pub(crate) fn fetch_one(&self) -> Result<R, oci::OracleError> {
        let mut iter = self.fetch_iter()?;
        match iter.next() {
            Some(v) => v,
            None => Err(oci::OracleError::new("The request returned no data".to_owned(), "statement.fetch_one"))
        }
    }

    fn get_result(&self, index: isize) -> ResultSet {
        let mut result = Vec::with_capacity(self.sizes.len());

        let mut v_offset: isize = 0;
        let mut i_offset = index;

        unsafe {
            for size in self.sizes.iter() {
                let offset = v_offset + size*index;
                let valp = self.values_p.offset(offset);

                let row_indicator = *self.indicators_p.offset(i_offset);
                let exists = row_indicator >= 0;

                let len = *self.ret_lengths_p.offset(i_offset);

                v_offset += size * self.prefetch_rows as isize;
                i_offset += self.prefetch_rows as isize;

                let value = if exists {
                    ResultValue::Val { valp, len }
                } else {
                    ResultValue::Nil
                };
                result.push(value);
            }
        }

        result
    }

    fn get_last_fetched_rows(&self) -> OracleResult<u32> {
        let mut rows_fetched: u32 = 0;
        let rows_fetcher_ptr: *mut u32 = &mut rows_fetched;

        oci::attr_get(self.stmthp as *mut oci::c_void, oci::OCI_HTYPE_STMT, rows_fetcher_ptr as *mut oci::c_void, oci::OCI_ATTR_ROWS_FETCHED, self.conn.errhp)?;
        Ok(rows_fetched)
    }

    fn initial_fetch(&self, success: bool) -> OracleResult<u32> {
        if success {
            Ok(self.prefetch_rows as u32)
        } else {
            // retrieve real count of fetched rows;
            self.get_last_fetched_rows()
        }
    }

    fn fetch(&self) -> Result<(u32, bool), oci::OracleError> {
        let mut done = false;

        if let Err(error) = oci::stmt_fetch(self.stmthp, self.conn.errhp, self.prefetch_rows as u32, oci::OCI_FETCH_NEXT, 0) {
            if error.errcode == 100 {
                /* OCI_NO_DATA */
                done = true;
            } else if error.errcode == 1406 {
                println!("WARNING: ORA-01406: Fetched column value was truncated!");
                done = true;
            } else {
                return Err(error);
            }
        }

        let rows_fetched = self.get_last_fetched_rows()?;
        Ok((rows_fetched, done))
    }
}

impl <R> Drop for ResultProcessor<'_,R> where R: ResultsProvider {
    fn drop(&mut self) {
        unsafe { dealloc(self.allocated_p, self.allocated_layout); };
    }
}

impl <'iter,'conn: 'iter, R> QueryIterator<'iter,'conn, R> where R: ResultsProvider {
    fn new(processor: &'iter ResultProcessor<'conn, R>, initial_prefetched: u32) -> QueryIterator<'iter,'conn, R> {
        QueryIterator { processor, done: false, initial_prefetched, rows_fetched: 0, cursor_index: 0, _result: PhantomData }
    }
}

impl <'iter, 'conn: 'iter, R> Iterator for QueryIterator<'iter,'conn, R> where R: ResultsProvider {
    type Item = oci::OracleResult<R>;

    fn next(&mut self) -> Option<oci::OracleResult<R>> {
        if self.done && self.cursor_index == 0 {
            return None;
        }

        if self.cursor_index == 0 {
            // println!("QueryIterator::cursor_index == 0");
            // need next fetch
            if self.rows_fetched == 0 {
                // println!("QueryIterator::need next fetch");
                // this is initial fetch because all next fetches set rows_fetched to real value
                if self.initial_prefetched == 0 {
                    return None;
                } else {
                    self.rows_fetched = self.initial_prefetched;
                    if self.rows_fetched < self.processor.prefetch_rows as u32 {
                        self.done = true;
                    }
                }
            } else {
                // subsequent fetches
                // println!("QueryIterator::subsequent fetches");
                match self.processor.fetch() {
                    Ok((rows_fetched, done)) => {
                        self.rows_fetched = rows_fetched;
                        self.done = done;
                    }
                    Err(err) => {
                        // error in iterator, close it
                        self.done = true;
                        self.cursor_index = 0;
                        // panic!("error in QueryIterator while fetch: {}", err);
                        // println!("QueryIterator::Error: {}", err);
                        return Some(Err(err));
                    }
                }
            }
        }

        // rows allready fetched, iterate over fetched rows
        if self.rows_fetched > 0 {
            // println!("QueryIterator::rows_fetched > 0");
            let result = self.processor.get_result(self.cursor_index as isize);
            self.cursor_index += 1;

            if self.cursor_index == self.rows_fetched {
                self.cursor_index = 0;
            }

            Some(Ok(R::from_resultset(&result)))
        } else {
            // println!("QueryIterator::rows_fetched == 0");
            None
        }
    }
}
