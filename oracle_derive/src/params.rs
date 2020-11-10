use proc_macro2::{Literal, Span, TokenStream};
use syn::{self, Data, Field, Ident, Index, Member, spanned::Spanned, Type, TypePath};
use quote::{quote, quote_spanned};

use crate::internals::Ctxt;
use crate::internals::ast::Container;
use std::convert::TryFrom;

/// Expands #[derive(Params)] macro.
pub fn expand_derive_params(input: &syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let ctxt = Ctxt::new();

    let cont = match Container::from_ast(&ctxt, input) {
        Some(cont) => cont,
        None => return Err(ctxt.check().unwrap_err()),
    };

    ctxt.check()?;

    let name = cont.ident;
    let (impl_generics, ty_generics, where_clause) = cont.generics.split_for_impl();

    let doc_comment = format!("Provide metainfo for `{}`.", name);

    let project_values_body = generate_project_values(&cont);
    let members_body = generate_members(&cont);

    Ok(quote! {
        impl #impl_generics oracle::ParamsProvider for #name #ty_generics #where_clause {
            #[doc = #doc_comment]
            fn project_values(&self, projecton: &mut oracle::ParamsProjection) {
                #project_values_body
            }

            #[doc = #doc_comment]
            fn members() -> Vec<oracle::Member> {
                #members_body
            }
        }

    })
}

/// Generate body of FromSqlValuesSet::from_values.
/// Work only for structs and tuples.
/// Example:
///         unsafe {
//             let p = projecton.get_unchecked_mut(0);
//             &self.id.project_value(p);
//         }
//  etc...
fn generate_project_values(cont: &Container) -> TokenStream {
    let expressions = cont.data.all_fields().enumerate().map(|(i,f)| {
        let index = Index::from(i);

        let member = match &f.member {
            Member::Named(name) => quote_spanned! { f.original.span() => #name },
            Member::Unnamed(_) => quote_spanned! { f.original.span() => #index },
        };

        quote_spanned! { f.original.span() =>
          unsafe {
            let p = projecton.get_unchecked_mut(#index);
            &self.#member.project_value(p);
          }
        }
    });
    quote! {
        #(#expressions);*
    }
}

/// Generate body of DescriptorsProvider::sql_descriptors.
/// Work only for structs and tuples.
/// Example:
///        use oracle::TypeDescriptorProducer;
//
//         vec![
//             oracle::Member::new(i32::produce(), oracle::Identifier::Named("id")),
//             oracle::Member::new(String::produce(), oracle::Identifier::Named("name")),
//         ]
// or for tuples:
//     use oracle::TypeDescriptorProducer;
//     vec![
//         oracle::Member::new(String::produce(), oracle::Identifier::Unnamed),
//         oracle::Member::new(String::produce(), oracle::Identifier::Unnamed),
//     ]
fn generate_members(cont: &Container) -> TokenStream {
    let expressions = cont.data.all_fields().map(|f| {
        let mut ty = f.ty;

        let convert_to_string =
            if let syn::Type::Reference(x) = ty {
                let ref_type = &x.elem;
                let path = extract_path(ref_type).expect("Can not parse type of field");
                let segment = path.path.segments.first().expect("Can not parse type of field");

                if segment.ident == "str" {
                    true
                } else {
                    false
                }
            } else {
                false
            };

        let producer = if convert_to_string {
            quote_spanned! { ty.span() => String::produce() }
        } else {
            quote_spanned! { ty.span() => #ty::produce() }
        };

        let ident = match &f.member {
            Member::Named(name) => {
                let name = name.to_string();
                // use stringify!($section) is invalid,
                // see: https://sequoia-pgp.gitlab.io/nettle-rs/quote/macro.quote.html
                quote_spanned! { f.original.span() => oracle::Identifier::Named(#name) }
            },
            Member::Unnamed(_) => quote_spanned! { f.original.span() => oracle::Identifier::Unnamed }
        };

        quote_spanned! { f.original.span() => oracle::Member::new(#producer, #ident) }
    });
    quote! {
        use oracle::TypeDescriptorProducer;
        vec![ #(#expressions),* ]
    }
}

fn extract_path(ty: &syn::Type) -> Option<&syn::TypePath> {
    if let syn::Type::Path(x) = ty {
        Some(x)
    } else {
        None
    }
}

fn extract_reference(ty: &syn::Type) -> Option<&syn::TypeReference> {
    if let syn::Type::Reference(x) = ty {
        Some(x)
    } else {
        None
    }
}

