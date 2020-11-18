use syn;
use crate::internals::ast::Field;

pub fn extract_column_size(field: &Field) -> Option<usize> {
    let ty = field.ty;

    let path = extract_path(ty).expect("Can not parse type of field");
    let segment = path.path.segments.first().expect("Can not parse type of field");

    if segment.ident == "String" {
        let attrs = &field.attrs;
        attrs.first().and_then(|a| {
            match a.parse_meta() {
                Ok(meta) =>
                    match (meta) {
                        syn::Meta::NameValue(nm) => {
                            if nm.path.segments.first().unwrap().ident != "col_size" {
                                panic!("Invalid attribute for String, must be: #[col_size=100]");
                            }

                            match nm.lit {
                                syn::Lit::Int(litint) => {
                                    let val = litint.base10_parse::<usize>().expect("Column attribute value must be integer literal, ex. #[col_size=100]");
                                    Some(val)
                                }
                                _ => {
                                    panic!("Column attribute value must be integer literal, ex. #[col_size=100]");
                                }
                            }
                        },
                        _ => {
                            panic!("Column attribute has invalid format, must be #[col_size=100]");
                        }
                    }
                Err(err) => {
                    panic!("Error parsing column attribute: {}, must be #[col_size=100]", err);
                }
            }
        }).take()
    } else {
        None
    }
}

pub fn extract_path(ty: &syn::Type) -> Option<&syn::TypePath> {
    if let syn::Type::Path(x) = ty {
        Some(x)
    } else {
        None
    }
}

pub fn extract_reference(ty: &syn::Type) -> Option<&syn::TypeReference> {
    if let syn::Type::Reference(x) = ty {
        Some(x)
    } else {
        None
    }
}
