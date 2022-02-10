use core::{
    fmt::{Display, Formatter, Result},
    ops::Index,
};

/// A slice you can index by bits
///
/// # Example
/// ```
/// // Assume `start` is a `*mut u8` that points to a valid region of memory and that `size` is its length
/// let mut bits = BitSlice::new();
/// unsafe { bits.init(start, size) }
///
/// bits.set[1] = true;
/// assert!(bits[1]);
/// ```
#[derive(Debug)]
pub struct BitSlice<'a> {
    pos: usize,
    pub data: &'a mut [u8],
}

impl<'a> BitSlice<'a> {
    /// Create a new BitSlice
    ///
    /// # Example
    /// ```
    /// // Assume `start` is a `*mut u8` that points to a valid region of memory and that `size` is its length
    /// let mut bits = BitSlice::new();
    /// unsafe { bits.init(start, size) }
    ///
    /// bits.set[1] = true;
    /// assert!(bits[1]);
    /// ```
    pub const fn new() -> Self {
        Self {
            pos: 0,
            data: &mut [],
        }
    }

    /// Initialize the BitSlice.
    /// This is unsafe because it is intended to be infallible.
    /// If it fails, behavior is undefined.
    ///
    /// # Arguments
    /// * `start` - A pointer to the data to use
    /// * `size` - The len of the data referenced by `start`
    ///
    /// # Example
    /// ```
    /// // Assume `start` is a `*mut u8` that points to a valid region of memory and that `size` is its length
    /// let mut bits = BitSlice::new();
    /// unsafe { bits.init(start, size) }
    ///
    /// bits.set[1] = true;
    /// assert!(bits[1]);
    /// ```
    pub unsafe fn init(&mut self, start: *mut u8, size: usize) {
        self.data = core::slice::from_raw_parts_mut(start, size);
        self.data.fill(0);
    }

    #[inline(always)]
    /// Calculate the needed numbers to get a certain bit
    ///
    /// # Arguments
    /// * `bit` - The desired bit
    fn calculate_offset(bit: usize) -> (usize, usize) {
        (bit / 8, bit % 8)
    }

    /// Set the specified bit
    ///
    /// # Arguments
    /// * `bit_set` - The bit to modify
    /// * `val` - The value to be moved into `index`
    /// # Example
    /// ```
    /// // Assume `start` is a `*mut u8` that points to a valid region of memory and that `size` is its length
    /// let mut bits = BitSlice::new();
    /// unsafe { bits.init(start, size) }
    ///
    /// bits.set[1] = true;
    /// assert!(bits[1]);
    /// ```
    pub fn set(&mut self, bit_set: usize, val: bool) {
        let (index, bit) = Self::calculate_offset(bit_set);
        // crate::peripherals::uart::println!("Set index {} bit {} to {}", index, bit, val);
        self.data[index] = (!val as u8 ^ self.data[index]) ^ (1 << bit);
    }

    /// Resets the internal position to 0, preparing the iterator to be traversed again
    pub fn reset_iterator(&mut self) {
        self.pos = 0;
    }
}

impl<'a> Index<usize> for BitSlice<'a> {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        let (index, bit) = Self::calculate_offset(index);

        if self.data[index] & (1 << bit) != 0 {
            &true
        } else {
            &false
        }
    }
}

impl<'a> Iterator for BitSlice<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.data.len() * 8 {
            let val = Some(self[self.pos]);
            self.pos += 1;
            val
        } else {
            None
        }
    }
}

impl<'a> Display for BitSlice<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        writeln!(f, "BitSlice {{").unwrap();
        for i in 0..self.data.len() * 8 {
            writeln!(f, "\t{i}: {},", self[i]).unwrap();
        }
        writeln!(f, "}}")
    }
}
