/// align initial size of row memory to align_value.
/// example align 2000, 1024 to 2048
pub fn align_size_to(initial_size: usize, align_value: usize) -> usize {
    let ceil = initial_size /  align_value;
    let remainder = initial_size % align_value;
    if remainder == 0 {
        initial_size
    } else {
        (ceil + 1) * align_value
    }
}