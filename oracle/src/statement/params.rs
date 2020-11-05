use std::alloc::{alloc, dealloc, Layout};
use std::marker::PhantomData;
use std::ptr::{null, null_mut};
use libc;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
use crate::oci;

use crate::connection::Connection;
use crate::types::TypeDescriptor;
use crate::statement::memory::align_size_to;

pub struct ParamValue {
    valp: *mut u8,
    indp: *mut i16,
    lenp: *mut u32,
}

pub struct Member {
    descriptor: TypeDescriptor,
    identifier: Identifier
}
pub enum Identifier {
    /// A named field like `self.x`.
    Named(String),
    /// An unnamed field like `self.0`.
    Unnamed,
}

pub type ParamsProjection = Vec<ParamValue>;

pub trait ParamsProvider {
    fn members() -> Vec<Member>;
    fn project_values(&self, projecton: &mut ParamsProjection) -> ();
}

/*
impl <'a> ParamValue {

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

    /// Convert optional type to row data
    #[inline]
    pub fn project_optional<U, F>(&mut self, param: Option<U>, f: F)
                                  -> Self where F: FnOnce(U) -> (*const u8, u16) {
        match param {
            None => ResultValue::Nil,
            Some(val) => Self::project(val, f)
        }
    }

    /// Convert non-optional type to row data
    #[inline]
    pub fn project<U, F>(param: U, f: F)
                         -> Self where F: FnOnce(U) -> (*const u8, u16) {
        let (valp, len) = f(param);
        ResultValue::Val{valp, len}
    }

}
*/

pub(crate) struct ParamsProcessor<'conn, P> where P: ParamsProvider {
    conn:    &'conn Connection,
    stmthp:  *mut oci::OCIStmt,

    sizes:            Vec<isize>,

    allocated_p:   *mut u8,    // pointer to a main allocated block
    allocated_layout: Layout,  // layout of allocated block

    values_p:         *const u8,            // pointer to values area
    indicators_p:     *const i16, // pointer to indicators area
    actual_lengths_p: *const u32,  // pointer to actual length area

    pub(crate) projection: ParamsProjection,

    _params: std::marker::PhantomData<P>
}

impl <'conn, P> ParamsProcessor<'conn, P> where P: ParamsProvider {
    pub(crate) fn new(conn: &'conn Connection, stmthp: *mut oci::OCIStmt) -> Result<ParamsProcessor<'conn, P>, oci::OracleError> {
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
                projection.push(ParamValue {valp: valp as *mut u8, indp: indp as *mut i16, lenp})
            }
        }

        Ok(ParamsProcessor {conn, stmthp, sizes, allocated_p, allocated_layout, values_p, indicators_p, actual_lengths_p, projection, _params: PhantomData})
    }

}

impl <P> Drop for ParamsProcessor<'_,P> where P: ParamsProvider {
    fn drop(&mut self) {
        unsafe { dealloc(self.allocated_p, self.allocated_layout); };
    }
}
