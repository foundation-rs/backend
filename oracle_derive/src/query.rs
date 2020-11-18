use proc_macro2::{Literal, Span, TokenStream};
use syn::{
    self,
    Data,
    Field,
    Ident,
    Index,
    Member,
    spanned::Spanned
};
use quote::{quote, quote_spanned};

use crate::internals::Ctxt;
use crate::internals::ast::Container;
use crate::utils::extract_column_size;

/// Expands #[derive(Query)] macro.
pub fn expand_derive_query(input: &syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let ctxt = Ctxt::new();

    let cont = match Container::from_ast(&ctxt, input) {
        Some(cont) => cont,
        None => return Err(ctxt.check().unwrap_err()),
    };

    ctxt.check()?;

    let name = cont.ident;
    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    let doc_comment = format!("Provide metainfo for `{}`.", name);

    let from_rs_body = generate_from_values(&cont);
    let descriptors_body = generate_descriptors_provider(&cont);

    Ok(quote! {
        impl #impl_generics oracle::ResultsProvider for #name #ty_generics #where_clause {
            #[doc = #doc_comment]
            fn from_resultset(rs: &oracle::ResultSet) -> Self {
                #from_rs_body
            }

            #[doc = #doc_comment]
            fn sql_descriptors() -> Vec<oracle::TypeDescriptor> {
                #descriptors_body
            }
        }

    })
}

/// Generate body of FromSqlValuesSet::from_values.
/// Work only for structs and tuples.
/// Example:
///         let s = ( &(rs[0]), &(rs[1]) );
///         OraTable { owner: s.0.into(), table_name: s.1.into() }
/// or for tuples:
///         OraTable ( s.0.into(), s.1.into() )
fn generate_from_values(cont: &Container) -> TokenStream {
    let expressions = cont.data.all_fields().enumerate().map(|(i,f)| {
        let index = Index::from(i);
        let body = quote_spanned! { f.original.span() => ( &(rs[#index]) ).into() };
        match &f.member {
            Member::Named(name) => quote_spanned! { f.original.span() => #name: #body },
            Member::Unnamed(_) => body
        }
    });
    let name = cont.ident;
    if cont.data.is_struct() {
        quote! {
        #name{ #(#expressions),* }
        }
    } else {
        quote! {
        #name( #(#expressions),* )
        }
    }
}

/// Generate body of DescriptorsProvider::sql_descriptors.
/// Work only for structs and tuples.
/// Example:
///        use oracle::TypeDescriptorProducer;
//
//         let type0 = String::produce_sized(128);
//         let type1 = String::produce_sized(128);
//
//         vec![type0, type1]
fn generate_descriptors_provider(cont: &Container) -> TokenStream {
    let expressions = cont.data.all_fields().map(|f| {
        let ty = f.ty;
        match extract_column_size(f) {
            Some(size) => {
                let size_literal = Literal::usize_unsuffixed(size);
                quote_spanned! { f.original.span() => #ty::produce_sized(#size_literal) }
            },
            None => quote_spanned! { f.original.span() => #ty::produce() }
        }
    });
    quote! {
        use oracle::TypeDescriptorProducer;
        vec![ #(#expressions),* ]
    }
}
