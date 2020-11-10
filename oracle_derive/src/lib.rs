extern crate quote;

#[macro_use]
extern crate syn;

extern crate proc_macro;
use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};
use quote::quote;

mod internals;
mod query;
mod params;

/// Generate Query implementation in form of #[derive(Query)]
/// example:
/// #[derive(Query)]
// pub struct OraTable {
//     owner: String,
//     table_name: String
// }
//
/// impl oracle::ResultsProvider for OraTable {
//     fn from_resultset(rs: &oracle::ResultSet) -> Self {
//         let s = ( &(rs[0]), &(rs[1]) );
//         OraTable { owner: s.0.into(), table_name: s.1.into() }
//     }

//     fn sql_descriptors() -> Vec<oracle::TypeDescriptor> {
//         use oracle::TypeDescriptorProducer;
//
//         let type0 = String::produce_sized(128);
//         let type1 = String::produce_sized(128);
//
//         vec![type0, type1]
//     }
// }
///
#[proc_macro_derive(Query)]
pub fn derive_query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    query::expand_derive_query(&input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

/// Generate Params implementation in form of #[derive(Params)]
/// example:
/// #[derive(Params)]
// pub struct OraTableColumnParams (String, String);
//
// impl oracle::ParamsProvider for OraTableColumnParams {
//    fn members() -> Vec<oracle::Member> {
//        use oracle::TypeDescriptorProducer;
//        vec![
//            oracle::Member::new(String::produce(), oracle::Identifier::Unnamed),
//            oracle::Member::new(String::produce(), oracle::Identifier::Unnamed),
//        ]
//    }
//
//    fn project_values(&self, projecton: &mut oracle::ParamsProjection) -> () {
//        unsafe {
//            let p = projecton.get_unchecked_mut(0);
//            &self.0.project_value(p);
//        }
//
//        unsafe {
//            let p = projecton.get_unchecked_mut(1);
//            &self.1.project_value(p);
//        }
//    }
//}
//
// pub struct TestingTuple2<'a> {
//    id: i32,
//    name: &'a str
//}
//
//impl <'a> oracle::ParamsProvider for TestingTuple2<'a> {
//    fn members() -> Vec<oracle::Member> {
//        use oracle::TypeDescriptorProducer;
//        vec![
//            oracle::Member::new(i32::produce(), oracle::Identifier::Named("id")),
//            oracle::Member::new(String::produce(), oracle::Identifier::Named("name")),
//        ]
//    }
//
//    fn project_values(&self, projecton: &mut oracle::ParamsProjection) -> () {
//        unsafe {
//            let p = projecton.get_unchecked_mut(0);
//            &self.id.project_value(p);
//        }
//
//        unsafe {
//            let p = projecton.get_unchecked_mut(1);
//            &self.name.project_value(p);
//        }
//    }
//}
#[proc_macro_derive(Params)]
pub fn derive_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    params::expand_derive_params(&input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}