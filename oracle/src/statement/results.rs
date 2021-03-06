use std::alloc::{alloc, dealloc, Layout};
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
#[derive(Debug, Copy, Clone)]
pub enum ResultValue {
    Val {
        valp: *const u8,
        len: u16,
    },
    Nil
}

pub type ResultSet = Vec<ResultValue>;

/// Trait for automatic processing of sql statement results.
/// Use `#[derive(SQLResults)]` for automatic implementation.
/// See `oracle_derive::SQLResults`
pub trait SQLResults {
    fn provider() -> Box<dyn ResultsProvider<Self>>;
}

pub trait ResultsProvider<T> {
    fn sql_descriptors(&self) -> Vec<TypeDescriptor>;
    fn gen_result(&self, rs: ResultSet) -> T;
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

pub(crate) struct ResultProcessor<'conn> {
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt,

    prefetch_rows: usize,
    sizes:         Vec<isize>,

    allocated_p:   *mut u8,    // pointer to a main allocated block
    allocated_layout: Layout,  // layout of allocated block

    values_p:      *const u8,  // pointer to values area
    indicators_p:  *const i16, // pointer to indicators area
    ret_lengths_p: *const u16, // pointer to return length area,
}

pub struct ResultIterator<'iter, 'conn: 'iter> {
    processor:          &'iter ResultProcessor<'conn>,
    done:               bool,
    initial_prefetched: u32,
    rows_fetched:       u32,
    cursor_index:       u32
}

impl <'conn> ResultProcessor<'conn> {

    pub(crate) fn new<'p, R>(conn: &'conn Connection, stmthp: *mut oci::OCIStmt, provider: &'p dyn ResultsProvider<R>, prefetch_rows: usize)
                      -> Result<ResultProcessor<'conn>, oci::OracleError> {
        let descriptors = provider.sql_descriptors();
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

        Ok( ResultProcessor {conn, stmthp, prefetch_rows, sizes, allocated_p, allocated_layout, values_p, indicators_p, ret_lengths_p} )
    }

    fn get_last_fetched_rows(&self) -> OracleResult<u32> {
        let mut rows_fetched: u32 = 0;
        let rows_fetcher_ptr: *mut u32 = &mut rows_fetched;

        oci::attr_get(self.stmthp as *mut oci::c_void, oci::OCI_HTYPE_STMT, rows_fetcher_ptr as *mut oci::c_void, oci::OCI_ATTR_ROWS_FETCHED, self.conn.errhp)?;
        Ok(rows_fetched)
    }

    pub (crate) fn fetch_iter<'iter> (&'conn self) -> OracleResult<ResultIterator<'iter, 'conn>> {
        let iters = self.prefetch_rows as u32;
        let success = oci::stmt_execute(self.conn.svchp, self.stmthp, self.conn.errhp, iters, 0)?;

        let initial_prefetched = 
            if success {
                Ok(self.prefetch_rows as u32)
            } else {
                // retrieve real count of fetched rows;
                self.get_last_fetched_rows()
            }?;

        // println!("initial prefetched: {}", initial_prefetched);

        Ok( ResultIterator::new(&self, initial_prefetched) )
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

    fn fetch_next(&self) -> OracleResult<(u32, bool)> {
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

impl Drop for ResultProcessor<'_> {
    fn drop(&mut self) {
        unsafe { dealloc(self.allocated_p, self.allocated_layout); };
    }
}

impl <'iter, 'conn: 'iter> ResultIterator<'iter, 'conn> {
    fn new(processor: &'conn ResultProcessor<'conn>, initial_prefetched: u32) -> ResultIterator<'iter, 'conn> {
        // println!("ResultIterator created");
        ResultIterator { processor, done: false, initial_prefetched, rows_fetched: 0, cursor_index: 0 }
    }

}

impl <'iter, 'conn: 'iter> Iterator for ResultIterator<'iter, 'conn> {
    type Item = oci::OracleResult<ResultSet>;

    fn next(&mut self) -> Option<oci::OracleResult<ResultSet>> {
        // println!("ResultIterator first fetch");
        if self.done && self.cursor_index == 0 {
            return None;
        }

        if self.cursor_index == 0 {
            // println!("ResultIterator::cursor_index == 0");
            // need next fetch
            if self.rows_fetched == 0 {
                // println!("ResultIterator::need next fetch");
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
                // println!("ResultIterator::subsequent fetches");
                match self.processor.fetch_next() {
                    Ok((rows_fetched, done)) => {
                        self.rows_fetched = rows_fetched;
                        self.done = done;
                    }
                    Err(err) => {
                        // error in iterator, close it
                        self.done = true;
                        self.cursor_index = 0;
                        // panic!("error in QueryIterator while fetch: {}", err);
                        // println!("ResultIterator::Error: {}", err);
                        return Some(Err(err));
                    }
                }
            }
        }

        // rows allready fetched, iterate over fetched rows
        if self.rows_fetched > 0 {
            // println!("ResultIterator::rows_fetched: {}", self.rows_fetched);
            let result = self.processor.get_result(self.cursor_index as isize);
            self.cursor_index += 1;

            if self.cursor_index == self.rows_fetched {
                self.cursor_index = 0;
            }

            // Some(Ok(self.processor.provider.gen_result(&result)))
            Some(Ok(result))
        } else {
            // println!("ResultIterator::rows_fetched == 0");
            None
        }
    }
}
