use syn;
use syn::punctuated::Punctuated;

use super::ctx::Ctxt;
use syn::token::Token;

/// A source data structure annotated with '#[derive(Query)]'
/// parsed into an internal representation.
pub struct Container<'a> {
    /// The struct or enum name (without generics).
    pub ident: &'a syn::Ident,
    /// The contents of the struct or enum.
    pub data: Data<'a>,
    /// Any generics on the struct or enum.
    pub generics: &'a syn::Generics,
    /// Original input
    pub original: &'a syn::DeriveInput,
}

/// The fields of a struct or enum.
/// Analogous to `syn::Data`.
pub enum Data<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(Style, Vec<Field<'a>>),
}

/// A variant of an enum.
pub struct Variant<'a> {
    pub ident: &'a syn::Ident,
    // pub attrs: attr::Variant,
    pub style: Style,
    pub fields: Vec<Field<'a>>,
    pub original: &'a syn::Variant,
}

/// A field of a struct
pub struct Field<'a> {
    pub member: syn::Member,
    // pub attrs: attr::Variant,
    pub ty: &'a syn::Type,
    pub original: &'a syn::Field,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Style {
    /// Named fields.
    Struct,
    /// Many unnamed fields.
    Tuple,
    /// One unnamed field.
    Newtype,
    /// No fields
    Unit,
}

impl <'a> Container <'a> {
    /// Convert the raw Syn ast into a parsed container object, collecting errors in `cs`.
    pub fn from_ast(
        cx: &Ctxt,
        item: &'a syn::DeriveInput
    ) -> Option<Container<'a>> {
        let data = match &item.data {
            syn::Data::Enum(data) => Data::Enum(enum_from_ast(cx, &data.variants)),
            syn::Data::Struct(data) => {
                let (style, fields) = struct_from_ast(cx, &data.fields);
                Data::Struct(style, fields)
            }
            syn::Data::Union(_) => {
                cx.error_spanned_by(item, "Oracle does not support query for unions");
                return None;
            }
        };

        let mut item = Container {
            ident: &item.ident,
            data,
            generics: &item.generics,
            original: item,
        };
        // check::check(cx, &mut item);
        Some(item)
    }
}

impl <'a> Data <'a> {
    pub fn all_fields(&'a self) -> Box<dyn Iterator<Item=&'a Field<'a>> + 'a> {
        match self {
            Data::Enum(variants) => {
                Box::new(variants.iter().flat_map(|variant| variant.fields.iter()))
            }
            Data::Struct(_, fields) => Box::new(fields.iter()),
        }
    }

    pub fn is_struct(&'a self) -> bool {
        if let Data::Struct(style,_) = self {
            *style == Style::Struct
        } else {
            false
        }
    }
}

fn enum_from_ast<'a>(
    cx: &Ctxt,
    variants: &'a Punctuated<syn::Variant, Token![,]>,
) -> Vec<Variant<'a>> {
    variants
        .iter()
        .map(|variant| {
            let (style, fields) = struct_from_ast(cx, &variant.fields);
            Variant {
                ident: &variant.ident,
                style,
                fields,
                original: variant,
            }
        })
        .collect()
}

fn struct_from_ast<'a>(
    cx: &Ctxt,
    fields: &'a syn::Fields,
) -> (Style, Vec<Field<'a>>) {
    match fields {
        syn::Fields::Named(fields) => (
            Style::Struct,
            fields_from_ast(cx, &fields.named),
        ),
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => (
            Style::Newtype,
            fields_from_ast(cx, &fields.unnamed),
        ),
        syn::Fields::Unnamed(fields) => (
            Style::Tuple,
            fields_from_ast(cx, &fields.unnamed),
        ),
        syn::Fields::Unit => (Style::Unit, Vec::new()),
    }
}

fn fields_from_ast<'a>(
    cx: &Ctxt,
    fields: &'a Punctuated<syn::Field, Token![,]>,
) -> Vec<Field<'a>> {
    fields
        .iter()
        .enumerate()
        .map(|(i, field)| Field {
            member: match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                None => syn::Member::Unnamed(i.into()),
            },
            ty: &field.ty,
            original: field,
        })
        .collect()
}
