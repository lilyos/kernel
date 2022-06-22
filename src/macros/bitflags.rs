use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

pub trait Bits:
    Clone + Copy + BitAnd + BitAndAssign + BitOr + BitOrAssign + BitXor + BitXorAssign + Not + Sized
{
    /// Where no bits are set
    const NONE: Self;

    /// Where all bits are set
    const ALL: Self;
}

macro_rules! impl_bits {
    ($($i:ty),*) => {
        $(
            impl Bits for $i {
                const NONE: $i = 0;

                const ALL: $i = <$i>::MAX;
            }
        )*
    }
}

impl_bits! {
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128
}

pub trait BitFlags {
    type BitType: Bits;

    /// Returns a zeroed bitflag struct
    fn none() -> Self;

    /// Returns a bitflag struct with all bits set
    fn all() -> Self;

    /// Returns the raw bit type of the bitflag struct
    fn bits(&self) -> Self::BitType;

    /// Get a bitflag struct from a set of raw bits where all bits correspond to
    /// a valid flag
    fn from_bits(bits: Self::BitType) -> Option<Self>
    where
        Self: Sized;

    /// Get a bitflag struct from a set of raw bits truncating unknown bits
    fn from_bits_truncate(bits: Self::BitType) -> Self;

    /// Get a bitflag struct from a set of raw bits, preserving everything
    unsafe fn from_bits_unchecked(bits: Self::BitType) -> Self;

    /// Returns if a bitflag struct is empty or not
    fn is_empty(&self) -> bool;

    /// Returns if a bitflag struct has all bits set or not
    fn is_all(&self) -> bool;

    /// Returns if a bitflag struct contains a specific flag
    fn contains(&self, other: Self) -> bool;

    /// Inserts a flag into a bitflag struct
    fn insert(&mut self, other: Self);

    /// Removes a flag from a bitflag struct
    fn remove(&mut self, other: Self);

    /// Toggles the given flag in the bitflag struct
    fn toggle(&mut self, other: Self);

    /// Sets the given flag in the bitflag struct to the provided value
    fn set(&mut self, other: Self, value: bool);
}

macro_rules! bitflags {
    (
        $(#[$outer_meta:meta])*
        $visibility:vis struct $bfs:ident: $T:ty {
            $(
                $(#[$inner_meta:ident $($args:tt)*])*
                const $Flag:ident = $val:expr;
            )*
        }
    ) => {
        $(#[$outer_meta])*
        #[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        /// The bitstruct
        $visibility struct $bfs {
            bits: $T,
        }

        $crate::macros::bitflags::impl_flags_bitflags! {
            $bfs: $T {
                $(
                    $(#[$inner_meta $($args)*])*
                    $Flag = $val;
                )*
            }
        }
    };
}

pub(crate) use bitflags;

macro_rules! impl_flags_bitflags {
    (
        $bfs:ident: $T:ty {
            $(
                $(#[$attr:ident $($args:tt)*])*
                $Flag:ident = $val:expr;
            )*
        }
    ) => {
        impl $bfs {
            $(
                $(#[$attr $($args)*])*
                /// The flag value
                pub const $Flag: Self = Self { bits: $val };
            )*

            /// Returns a zeroed flag struct
            #[inline]
            pub const fn none() -> Self {
                Self {
                    bits: <$T as $crate::macros::bitflags::Bits>::NONE,
                }
            }

            /// Returns a maxed flag struct
            #[inline]
            pub const fn all() -> Self {
                Self {
                    bits: <$T as $crate::macros::bitflags::Bits>::ALL,
                }
            }

            /// Returns the raw bits of the struct
            #[inline]
            pub const fn bits(&self) -> $T {
                self.bits
            }

            /// Tries to convert the given bits to the struct, returning none if any are unknown
            #[inline]
            pub const fn from_bits(bits: $T) -> ::core::option::Option<Self> {
                let truncated = Self::from_bits_truncate(bits).bits;

                if truncated == bits {
                    ::core::option::Option::Some(Self { bits })
                } else {
                    ::core::option::Option::None
                }
            }

            /// Makes the struct by truncating unknown bits
            #[inline]
            pub const fn from_bits_truncate(bits: $T) -> Self {
                if bits == <$T as $crate::macros::bitflags::Bits>::NONE {
                    return Self { bits };
                }

                let mut builder = <$T as $crate::macros::bitflags::Bits>::NONE;

                $(
                    #[allow(unused_doc_comments)]
                    $(#[$attr $($args)*])*
                    if bits & Self::$Flag.bits == Self::$Flag.bits {
                        builder |= Self::$Flag.bits
                    }
                )*

                Self {
                    bits: builder
                }
            }

            /// Makes a struct from the bits without truncating
            ///
            /// # Safety
            /// The caller must make sure that the unused bits do not violate any contract from the bit struct type
            #[inline]
            pub const unsafe fn from_bits_unchecked(bits: $T) -> Self {
                Self { bits }
            }

            /// Returns if the bitstruct is empty or not
            #[inline]
            pub const fn is_empty(&self) -> bool {
                self.bits() == Self::none().bits()
            }

            /// Returns if the bitstruct is fully set or not
            #[inline]
            pub const fn is_all(&self) -> bool {
                Self::all().bits() | self.bits == self.bits
            }

            /// Checks if the bitstruct contains a given flag or set of flags
            #[inline]
            pub const fn contains(&self, other: Self) -> bool {
                (self.bits & other.bits) == other.bits
            }

            /// Inserts a flag or set of flags into the bistruct
            #[inline]
            pub fn insert(&mut self, other: Self) {
                self.bits |= other.bits;
            }

            /// Removes a flag or set of flags from the bitstruct
            #[inline]
            pub fn remove(&mut self, other: Self) {
                self.bits &= !other.bits;
            }

            /// Toggles a given flag or set of flags for the bitstruct
            #[inline]
            pub fn toggle(&mut self, other: Self) {
                self.bits ^= other.bits;
            }

            /// Sets the given flag or sets of flags to a given value
            #[inline]
            pub fn set(&mut self, other: Self, val: bool) {
                if val {
                    self.insert(other);
                } else {
                    self.remove(other);
                }
            }

            #[allow(dead_code, unused_doc_comments, unused_attributes)]
            /// Returns an interator for the bitstruct
            pub fn iter(self) -> impl ::core::iter::Iterator<Item = (&'static str, Self)> {
                use ::core::iter::Iterator as _;

                const FLAG_NUM: usize = {
                    let mut flag_num = 0;

                    $(
                        $(#[$attr $($args)*])*
                        {
                            flag_num += 1;
                        }
                    )*

                    flag_num
                };

                const OPTIONS: [$bfs; FLAG_NUM] = [
                    $(
                        $(#[$attr $($args)*])*
                        $bfs::$Flag,
                    )*
                ];

                const OPTIONS_NAMES: [&'static str; FLAG_NUM] = [
                    $(
                        $(#[$attr $($args)*])*
                        ::core::stringify!($Flag),
                    )*
                ];

                let mut start = 0;
                let mut state = self;
                ::core::iter::from_fn(move || {
                    if state.is_empty() || FLAG_NUM == 0 {
                        ::core::option::Option::None
                    } else {
                        for (flag, flag_name) in OPTIONS[start..FLAG_NUM].iter().copied().zip(OPTIONS_NAMES[start..FLAG_NUM].iter().copied()) {
                            start += 1;

                            if self.contains(flag) {
                                state.remove(flag);

                                return ::core::option::Option::Some((flag_name, flag));
                            }
                        }

                        core::option::Option::None
                    }
                })
            }


        }
        impl ::core::ops::BitOr for $bfs {
            type Output = Self;

            #[inline]
            fn bitor(self, other: Self) -> Self {
                Self { bits: self.bits | other.bits }
            }
        }

        impl ::core::ops::BitOrAssign for $bfs {
            #[inline]
            fn bitor_assign(&mut self, other: Self) {
                self.bits |= other.bits;
            }
        }

        impl ::core::ops::BitXor for $bfs {
            type Output = Self;

            #[inline]
            fn bitxor(self, other: Self) -> Self {
                Self { bits: self.bits ^ other.bits }
            }
        }

        impl ::core::ops::BitXorAssign for $bfs {
            #[inline]
            fn bitxor_assign(&mut self, other: Self) {
                self.bits ^= other.bits;
            }
        }

        impl ::core::ops::BitAnd for $bfs {
            type Output = Self;

            #[inline]
            fn bitand(self, other: Self) -> Self {
                Self { bits: self.bits & other.bits }
            }
        }

        impl ::core::ops::BitAndAssign for $bfs {
            #[inline]
            fn bitand_assign(&mut self, other: Self) {
                self.bits &= other.bits;
            }
        }

        impl ::core::ops::Sub for $bfs {
            type Output = Self;

            #[inline]
            fn sub(self, other: Self) -> Self {
                Self { bits: self.bits & !other.bits }
            }
        }

        impl ::core::ops::SubAssign for $bfs {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                self.bits &= !other.bits;
            }
        }

        impl ::core::ops::Not for $bfs {
            type Output = Self;

            #[inline]
            fn not(self) -> Self {
                Self { bits: !self.bits } & Self::all()
            }
        }

        impl ::core::iter::Extend<$bfs> for $bfs {
            fn extend<T: ::core::iter::IntoIterator<Item=Self>>(&mut self, iterator: T) {
                for item in iterator {
                    self.insert(item)
                }
            }
        }

        impl ::core::iter::FromIterator<$bfs> for $bfs {
            fn from_iter<T: ::core::iter::IntoIterator<Item=Self>>(iterator: T) -> Self {
                use ::core::iter::Extend;

                let mut result = Self::none();
                result.extend(iterator);
                result
            }
        }

        impl $crate::macros::bitflags::BitFlags for $bfs {
            type BitType = $T;

            fn none() -> Self {
                $bfs::none()
            }

            fn all() -> Self {
                $bfs::all()
            }

            fn bits(&self) -> $T {
                $bfs::bits(self)
            }

            fn from_bits(bits: $T) -> ::core::option::Option<Self> {
                $bfs::from_bits(bits)
            }

            fn from_bits_truncate(bits: $T) -> Self {
                $bfs::from_bits_truncate(bits)
            }

            unsafe fn from_bits_unchecked(bits: $T) -> Self {
                $bfs::from_bits_unchecked(bits)
            }

            fn is_empty(&self) -> bool {
                $bfs::is_empty(self)
            }

            fn is_all(&self) -> bool {
                $bfs::is_all(self)
            }

            fn contains(&self, other: Self) -> bool {
                $bfs::contains(self, other)
            }

            fn insert(&mut self, other: Self) {
                $bfs::insert(self, other)
            }

            fn remove(&mut self, other: Self) {
                $bfs::remove(self, other)
            }

            fn toggle(&mut self, other: Self) {
                $bfs::toggle(self, other)
            }

            fn set(&mut self, other: Self, val: bool) {
                $bfs::set(self, other, val)
            }
        }
    };
}

pub(crate) use impl_flags_bitflags;
