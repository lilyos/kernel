/// Aligns a given address to a specified power of two (upwards)
///
/// # Arguments
/// * `addr` - The address to align
/// * `align` - The power of two to align to
pub fn align(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Aligns a given address to a specified power of two (downwards)
///
/// # Arguments
/// * `addr` - The address to align
/// * `align` - The power of two to align to
pub fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

pub fn mask_low_bits(size: usize) -> usize {
    !0 >> size << size
}

pub fn is_address_canonical(addr: usize, width: usize) -> bool {
    let mask = mask_low_bits(width - 1);
    (addr & mask) == mask || (addr & mask) == 0
}
