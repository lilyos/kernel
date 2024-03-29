/// Aligns a given usize to a specified power of two (upwards)
///
/// # Arguments
/// * `addr` - The address to align
/// * `align` - The power of two to align to
#[must_use]
pub const fn align(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Aligns a given usize to a specified power of two (downwards)
///
/// # Arguments
/// * `addr` - The address to align
/// * `align` - The power of two to align to
#[must_use]
pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Get the highest `size` bits
///
/// # Arguments
/// * `size` - The size of the mask in bits
#[must_use]
pub const fn mask_low_bits(size: usize) -> usize {
    !0 >> size << size
}

/// Tell if an address is canonical
///
/// # Arguments
/// * `addr` - The address to check
/// * `width` - The width of the pointer
#[must_use]
pub const fn is_address_canonical(addr: usize, width: usize) -> bool {
    let mask = mask_low_bits(width - 1);
    (addr & mask) == mask || (addr & mask) == 0
}
