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
use std::cell::RefCell;

pub struct ParamValue {
    valp: *mut u8,
    indp: *mut i16,
    lenp: *mut u32,
    size: usize
}

pub struct Member {
    descriptor: TypeDescriptor,
    identifier: Identifier
}
pub enum Identifier {
    /// A named field like `self.x`.
    Named(&'static str),
    /// An unnamed field like `self.0`.
    Unnamed,
}

pub type ParamsProjection = Vec<ParamValue>;

/// Trait for automatic processing of sql statement parameters
pub trait ParamsProvider {
    fn members() -> Vec<Member>;
    fn project_values(&self, projecton: &mut ParamsProjection) -> ();
}

pub trait ValueProjector<T> {
    fn project_value(&self, projection: &mut ParamValue);
}

impl Member {
    pub fn new(descriptor: TypeDescriptor,identifier: Identifier) -> Self {
        Member { descriptor, identifier }
    }
}

impl <'a> ParamValue {

    /// Convert optional type to row data
    #[inline]
    pub fn project_optional<U, F>(&mut self, param: &Option<U>, f: F)
                                  -> () where F: FnOnce(*mut u8, *mut i16) -> usize {
        unsafe {
            match param {
                None => {
                    *self.indp = -1;
                },
                Some(val) => {
                    *self.indp = 0;
                    self.project(val, f);
                }
            }
        };
    }

    /// Convert non-optional type to row data
    #[inline]
    pub fn project<U, F>(&mut self, _param: &U, f: F)
                         -> () where F: FnOnce(*mut u8, *mut i16) -> usize {
        unsafe {
            *self.indp = 0;
            let actual_size = f(self.valp, self.indp);
            if *self.indp == 0 {
                if actual_size > 0 && actual_size <= self.size {
                    *self.lenp = actual_size as u32;
                } else {
                    *self.lenp = self.size as u32;
                }
            }
        }
    }

}

pub(crate) struct ParamsProcessor<P> where P: ParamsProvider {
    allocated_p:   *mut u8,    // pointer to a main allocated block
    allocated_layout: Layout,  // layout of allocated block

    pub(crate) projection: RefCell<ParamsProjection>,

    _params: std::marker::PhantomData<P>
}

impl <P> ParamsProcessor<P> where P: ParamsProvider {
    pub(crate) fn new(conn: &Connection, stmthp: *mut oci::OCIStmt) -> Result<ParamsProcessor<P>, oci::OracleError> {
        let members = P::members();
        let columns_cnt = members.len();

        let val_size = members.iter().map(|m| m.descriptor.size ).sum::<usize>();
        let area_size = align_size_to(val_size, 128);
        let inds_size = align_size_to(2 * columns_cnt, 64);
        let lens_size = align_size_to(4 * columns_cnt, 64);

        let total_size = align_size_to(area_size + inds_size + lens_size, 256);

        let allocated_layout = Layout::array::<u8>(total_size).unwrap();
        let allocated_p = unsafe { alloc(allocated_layout) };

        if allocated_p.is_null() {
            panic!("failed to allocate memory for Parameters buffer");
        }

        let indicators_p = allocated_p as *const i16;
        let actual_lengths_p = unsafe { allocated_p.offset(inds_size as isize) } as *const u32;
        let values_p = unsafe { allocated_p.offset((inds_size + lens_size) as isize) } as *const u8;

        let mut offset = 0;
        let mut offset_i = 0;

        let mut sizes = Vec::with_capacity(columns_cnt);
        let mut projection = Vec::with_capacity(columns_cnt);

        unsafe {
            for (i,m) in members.iter().enumerate() {
                let d = &m.descriptor;
                let valp = values_p.offset(offset) as *mut libc::c_void;
                let indp = indicators_p.offset(offset_i) as *mut libc::c_void;
                let lenp = actual_lengths_p.offset(offset_i) as *mut u32;

                offset += d.size as isize;
                offset_i += 1 as isize;

                match &m.identifier {
                    Identifier::Named(name) => {
                        oci::bind_by_name(stmthp, conn.errhp, name, valp, indp, d.size as i64, lenp, d.dtype)?;
                    },
                    Identifier::Unnamed => {
                        oci::bind_by_pos(stmthp, conn.errhp, (i+1) as u32, valp, indp, d.size as i64, lenp, d.dtype)?;
                    }
                }

                sizes.push(d.size as isize);
                projection.push(ParamValue {valp: valp as *mut u8, indp: indp as *mut i16, lenp, size: d.size})
            }
        }

        let projection = RefCell::new(projection);

        Ok(ParamsProcessor {allocated_p, allocated_layout, projection, _params: PhantomData})
    }

}

impl <P> Drop for ParamsProcessor<P> where P: ParamsProvider {
    fn drop(&mut self) {
        unsafe { dealloc(self.allocated_p, self.allocated_layout); };
    }
}
