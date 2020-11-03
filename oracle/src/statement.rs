use std::marker::PhantomData;
use libc;

use crate::connection::Connection;
use crate::types::{
    DescriptorsProvider,
    TypeDescriptor
};
use crate::values::{
    FromResultSet,
    ResultSet,
    SqlValue
};

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

/// Generic prepared statement
pub struct Statement<'conn> {
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt,
}

/// Statement with parameters (bindings)
pub struct BindedStatement<'conn, P> where P: DescriptorsProvider {
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt,
    _params: std::marker::PhantomData<P>
}

/// Statement with ResultSet (defined result)
pub struct Query<'conn,R> where R: DescriptorsProvider + FromResultSet {
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt,
    fetcher: Option<Box<Fetcher<'conn>>>,
    _result: std::marker::PhantomData<R>
}

/// Statement with parameters (bindings) and ResultSet (defined result)
pub struct BindedQuery<'conn,R, P>
    where R: DescriptorsProvider + FromResultSet,
          P: DescriptorsProvider
{
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt,
    fetcher: Option<Box<Fetcher<'conn>>>,
    _result: std::marker::PhantomData<R>,
    _params: std::marker::PhantomData<P>
}

struct Fetcher<'conn> {
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt,
    prefetch_rows: usize,
    sizes:         Vec<isize>,

    values_p:      *const u8,             // pointer to values area
    indicators_p:  *const libc::c_short,  // pointer to indicators area
    ret_lengths_p: *const libc::c_ushort, // pointer to return length area
}

pub struct QueryIterator<'iter,'conn: 'iter, R> where R: FromResultSet {
    fetcher: &'iter mut Fetcher<'conn>,
    done:    bool,
    rows_fetched: u32,
    cursor_index: u32,
    _result: std::marker::PhantomData<R>
}

impl <'conn> Statement<'conn> {
    pub(crate) fn new<'s>(conn: &'conn Connection, sql:  &'s str) -> Result<Statement<'conn>, oci::OracleError> {
        let stmthp = oci::stmt_prepare(conn.svchp, conn.errhp, sql)?;
        Ok( Statement { conn, stmthp } )
    }

    /// Prepare oracle statement
    pub fn define<R: DescriptorsProvider + FromResultSet>(self) -> Query<'conn,R> {
        Query::new(self)
    }

    /// Execute generic statement
    pub fn execute(&mut self) -> Result<(),oci::OracleError> {
        oci::stmt_execute(self.conn.svchp, self.stmthp, self.conn.errhp, 0, 0)?;
        Ok(())
    }

}

impl <'conn,R> Query<'conn,R> where R: DescriptorsProvider + FromResultSet {
    fn new<'s>(statement: Statement<'s>) -> Query<'s,R> {
        let conn = statement.conn;
        let stmthp = statement.stmthp;
        Query { conn, stmthp, fetcher: None, _result: PhantomData }
    }

    fn inner_prefetch_rows(&mut self, prefetch_rows: usize) -> Result<(), oci::OracleError> {
        match & self.fetcher {
            None => {
                let fetcher = Fetcher::new(self.conn, self.stmthp, prefetch_rows, R::sql_descriptors())?;
                self.fetcher = Some(Box::new(fetcher));
                Ok(())
            }
            Some(f) => {
                if f.prefetch_rows == prefetch_rows {
                    Ok(())
                } else {
                    Err(oci::OracleError::new("prefetch_rows allready set with different number of rows".to_owned(),"statement::prefetch_rows"))
                }
            }
        }
    }

    pub fn prefetch_rows(mut self, prefetch_rows: usize) -> Result<Self, oci::OracleError> {
        self.inner_prefetch_rows(prefetch_rows)?;
        Ok(self)
    }

    pub fn fetch_iter<'iter>(&'iter mut self) -> Result<QueryIterator<'iter, 'conn, R>, oci::OracleError> {
        self.execute(10)?;
        Ok(QueryIterator::new(self.fetcher.as_mut().unwrap().as_mut()))
    }

    pub fn fetch_list(&mut self) -> Result<Vec<R>, oci::OracleError> {
        let capacity = match self.fetcher.as_ref() {
            Some(f) => f.prefetch_rows,
            None => 10
        };
        let mut result = Vec::with_capacity(capacity);

        for v in self.fetch_iter()? {
            match v {
                Ok(v) => result.push(v),
                Err(err) => return Err(err)
            };
        }
        Ok( result )
    }

    pub fn fetch_one(&mut self) -> Result<R, oci::OracleError> {
        self.inner_prefetch_rows(1)?;
        let mut iter = self.fetch_iter()?;
        match iter.next() {
            Some(v) => v,
            None => Err(oci::OracleError::new("The request returned no data".to_owned(), "statement.fetch_one"))
        }
    }

    fn execute(&mut self, prefetch_rows: usize) -> Result<(),oci::OracleError> {
        if let None = self.fetcher {
            let fetcher = Fetcher::new(self.conn, self.stmthp, prefetch_rows, R::sql_descriptors()).unwrap();
            self.fetcher = Some (Box::new(fetcher));
        };

        oci::stmt_execute(self.conn.svchp, self.stmthp, self.conn.errhp, 0, 0)?;
        Ok(())
    }

}

impl <R> Drop for Query <'_,R> where R: DescriptorsProvider + FromResultSet {
    fn drop(&mut self) {
        oci::stmt_release(self.stmthp, self.conn.errhp);
    }
}

impl <'conn> Fetcher<'conn> {
    fn new(conn: &'conn Connection, stmthp: *mut oci::OCIStmt, prefetch_rows: usize, descriptors: Vec<TypeDescriptor>) -> Result<Fetcher, oci::OracleError> {
        let columns_cnt = descriptors.len();
        let val_size = descriptors.iter().map(|d| d.size ).sum::<usize>();
        let area_size = val_size * prefetch_rows;
        let inds_size = 2 * columns_cnt * prefetch_rows;

        let values_p = unsafe { libc::malloc(area_size) as *const u8 };
        let indicators_p = unsafe { libc::malloc(inds_size) as *const libc::c_short };
        let ret_lengths_p = unsafe { libc::malloc(inds_size) as *const libc::c_ushort };

        if values_p.is_null() || indicators_p.is_null() || ret_lengths_p.is_null() {
            panic!("failed to allocate mem for Cursor");
        }

        let mut offset = 0;
        let mut offset_i = 0;

        let mut sizes = Vec::with_capacity(columns_cnt);

        unsafe {
            for (i,d) in descriptors.iter().enumerate() {
                let value_p = values_p.offset(offset) as *mut libc::c_void;
                let ind_p = indicators_p.offset(offset_i) as *mut libc::c_void;
                let rlen_p = ret_lengths_p.offset(offset_i) as *mut u16;

                offset += (d.size * prefetch_rows) as isize;
                offset_i += prefetch_rows as isize;

                oci::define_by_pos(stmthp, conn.errhp, (i+1) as u32, value_p, ind_p, d.size as i32, rlen_p, d.dtype)?;

                sizes.push(d.size as isize);
            }
        }

        Ok(Fetcher {conn, stmthp, prefetch_rows, sizes, values_p, indicators_p, ret_lengths_p})
    }

    fn result(&mut self, index: isize) -> ResultSet {
        let mut result = Vec::with_capacity(self.sizes.len());

        let mut v_offset: isize = 0;
        let mut i_offset = index;

        unsafe {
            for size in self.sizes.iter() {
                let offset = v_offset + size*index;
                let val_p = self.values_p.offset(offset);

                let row_indicator = *self.indicators_p.offset(i_offset);
                let exists = row_indicator >= 0;

                let len = *self.ret_lengths_p.offset(i_offset);

                v_offset += size * self.prefetch_rows as isize;
                i_offset += self.prefetch_rows as isize;

                let value = if exists {
                    SqlValue::new ( val_p, len )
                } else {
                    SqlValue::Nil
                };
                result.push(value);
            }
        }

        result
    }

    fn fetch(&mut self) -> Result<(u32, bool), oci::OracleError> {
        let mut done = false;

        if let Err(error) = oci::stmt_fetch(self.stmthp, self.conn.errhp, self.prefetch_rows as u32, oci::OCI_FETCH_NEXT, 0) {
            if error.errcode == 100 {
                done = true;
            } else if error.errcode == 1406 {
                println!("WARNING: ORA-01406: Fetched column value was truncated!");
                done = true;
            } else {
                return Err(error);
            }
        }

        let mut rows_fetched: u32 = 0;
        let rows_fetcher_ptr: *mut u32 = &mut rows_fetched;

        oci::attr_get(self.stmthp as *mut oci::c_void, oci::OCI_HTYPE_STMT, rows_fetcher_ptr as *mut oci::c_void, oci::OCI_ATTR_ROWS_FETCHED, self.conn.errhp)?;

        Ok((rows_fetched, done))
    }
}

impl Drop for Fetcher<'_> {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.values_p as *mut libc::c_void);
            libc::free(self.indicators_p as *mut libc::c_void);
            libc::free(self.ret_lengths_p as *mut libc::c_void);
        }
    }
}

impl <'iter,'conn: 'iter, R> QueryIterator<'iter,'conn, R> where R: FromResultSet {
    fn new(fetcher: &'iter mut Fetcher<'conn>) -> QueryIterator<'iter,'conn, R> {
        QueryIterator { fetcher: fetcher, done: false, rows_fetched: 0, cursor_index: 0, _result: PhantomData }
    }
}

impl <'iter, 'conn: 'iter, R> Iterator for QueryIterator<'iter,'conn, R> where R: FromResultSet {
    type Item = oci::OracleResult<R>;

    fn next(&mut self) -> Option<oci::OracleResult<R>> {
        if self.done && self.cursor_index == 0 {
            return None;
        }

        if self.cursor_index == 0 {
            match self.fetcher.fetch() {
                Ok((rows_fetched, done)) => {
                    self.rows_fetched = rows_fetched;
                    self.done = done;
                }
                Err(err) => {
                    // error in iterator, close it
                    self.done = true;
                    self.cursor_index = 0;
                    // panic!("error in QueryIterator while fetch: {}", err);
                    return Some(Err(err));
                }
            }
        }

        if self.rows_fetched > 0 {
            let result = self.fetcher.result(self.cursor_index as isize);
            self.cursor_index += 1;

            if self.cursor_index == self.rows_fetched {
                self.cursor_index = 0;
            }
            Some(Ok(R::from_resultset(&result)))
        } else {
            None
        }
    }
}

