/// Aligns a given address to a specified power of two
///
/// # Arguments
/// * `addr` - The address to align
/// * `align` - The power of two to align to
pub fn align(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
