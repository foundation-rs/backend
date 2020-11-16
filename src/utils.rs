/// if optional doesn't have value, then call creator, put new value and return it
/// if optional does have value, check it with predicate
/// if predicate return true, return old value
/// if predicate return false, call creator and return new valuue, return old value
pub fn get_or_insert_with_condition<T,F,P>(optional: &mut Option<T>, creator: F, predicate: P) 
-> (&mut T, Option<T>) where F: FnOnce() -> T, 
                             P: FnOnce(&T) -> bool {
    use std::hint;
    use std::mem;

    let old = 
    if let Some(ref mut v) = *optional {
        if predicate(&v) {
            None
        } else {
            // extract old value and replace it with new value
            Some(mem::replace(v, creator()))
        }
    } else {
        *optional = Some(creator());
        None
    };

    match optional {
        Some(v) => (v,old),
        // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
        // variant in the code above.
        None => unsafe { hint::unreachable_unchecked() },
    }
}