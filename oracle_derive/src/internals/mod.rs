pub mod ast;
mod ctx;

pub use self::ctx::Ctxt;

use syn::Type;

pub fn ungroup(mut ty: &Type) -> &Type {
    // If a type contained within invisible delimiters.
    while let Type::Group(group) = ty {
        ty = &group.elem;
    }
    ty
}