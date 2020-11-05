#[macro_use]
extern crate quote;

#[macro_use]
extern crate syn;

extern crate proc_macro;
use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};
use quote::quote;

mod internals;
mod query;

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

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}