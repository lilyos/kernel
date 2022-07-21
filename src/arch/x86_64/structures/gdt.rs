use core::{
    arch::asm,
    fmt::Debug,
    ops::{Index, IndexMut},
};

use crate::macros::bitflags::bitflags;

/*
/// Test if we're in 64_bit mode
pub fn is_64_bit_mode() -> bool {
    let out: u8;
    unsafe {
        asm!(
            "jmp 4f",
            "2:",
            "mov {0}, 0x0",
            "3:",
            "mov {0}, 0x1",
            "4:",
            "mov eax, 0x80000000",
            "cpuid",
            "cmp eax, 0x80000001",
            "jb 2b",
            "mov eax, 0x80000001",
            "cpuid",
            "test edx, 1 << 29",
            "jz 2b",
            "jmp 3b",
            out(reg_byte) out,
            out("eax") _,
            out("edx") _,
        )
    }
    out != 0
}
*/

/// This is okay since Limine drops us into 64 bit mode
pub fn is_64_bit_mode() -> bool {
    true
}

/// The kernel's GDT
#[used]
pub static mut GDT: [u64; 9] = [
    0,
    // 0b0000_0000_0010_0000_1001_1010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
    // 0b0000_0000_0100_0000_1001_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
    // 0b0000_0000_0100_0000_1111_1010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
    // 0b0000_0000_0100_0000_1111_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
    // 0b0000_0000_0010_0000_1111_1010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
    // 0b0000_0000_0100_0000_1111_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
    0x0000000000209A00,
    0x0000000000009200,
    0x000000000040FA00,
    0x000000000040F200,
    0x000000000020FA00,
    0x000000000000F200,
    0,
    0,
];

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
/// Segment types for 32 bit mode
pub enum SegmentType32Bit {
    /// 16 bit Tss (Available)
    Tss16BitAvailable = 0x1,
    /// Local Descriptor Table
    Ldt = 0x2,
    /// 16 bit Tss (Busy)
    Tss16BitBusy = 0x3,
    /// 32 bit Tss (Available)
    Tss32BitAvailable = 0x9,
    /// 32 bit Tss (Busy)
    Tss32BitBusy = 0xB,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
/// Segment types for 64 bit mode
pub enum SegmentType64Bit {
    /// Local Descriptor Table
    Ldt = 0x2,
    /// 64 bit Tss (Available)
    Tss64BitAvailable = 0x9,
    /// 64 bit Tss (Busy)
    Tss64BitBusy = 0xB,
}

/// Union of segment types
pub union SegmentType {
    /// Protected Mode Segment Type
    pub protected: SegmentType32Bit,
    /// Long Mode Segment Type
    pub long: SegmentType64Bit,
    untyped: u8,
}

impl Debug for SegmentType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe {
            if is_64_bit_mode() {
                <SegmentType64Bit as Debug>::fmt(&self.long, f)
            } else {
                <SegmentType32Bit as Debug>::fmt(&self.protected, f)
            }
        }
    }
}

bitflags! {
    pub struct CodeDataSegmentAccessByte: u8 {
        /// Set if the segment is accessed
        const ACCESSED = 1 << 0;

        /// Read/Write options
        ///
        /// # For Code Segments
        /// If clear, reading is not allowed.
        /// If set (1), reading is allowed. Writing is always forbidden.
        ///
        /// # For Data Segments
        /// If clear, writing is not allowed.
        /// If set (1), writing is allowed. Reading is always allowed.
        const READ_WRITE = 1 << 1;

        /// Direction bit/Conforming bit
        ///
        /// # For Code Segments
        /// If clear, this segment can only be executed from the specified privilege level.
        /// If set, this segment can be executed from the specified privilege level or lower.
        ///
        /// # For Data Segments
        /// If clear, this segment grows upwards.
        /// If set, this segment grows downwards.
        const DIRECTION_CONFORMING = 1 << 2;

        /// Excutable Bit
        ///
        /// If clear, the descriptor defines a data segment.
        /// If set, the descriptor defines a code segment.
        const EXECUTABLE = 1 << 3;

        /// The type of descriptor.
        ///
        /// If clear, this is a System Segment.
        /// If set, this is a code or data segment.
        const CODE_DATA_SEGMENT = 1 << 4;

        /// If the segment is present
        const PRESENT = 1 << 7;
    }
}

impl CodeDataSegmentAccessByte {
    /// Set descriptor privilege level
    pub const fn descriptor_privilege_level(mut self, level: u8) -> Self {
        self.bits |= (level & 0x2) << 6;
        self
    }

    /// Get descriptor privilege level
    pub const fn get_descriptor_privilege_level(&self) -> u8 {
        (self.bits >> 6) & 0x2
    }
}

impl Debug for CodeDataSegmentAccessByte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CodeDataSegmentAccessByte")
            .field("Accessed", &self.contains(Self::ACCESSED))
            .field("ReadWrite", &self.contains(Self::READ_WRITE))
            .field(
                "DirectionConforming",
                &self.contains(Self::DIRECTION_CONFORMING),
            )
            .field("Executable", &self.contains(Self::EXECUTABLE))
            .field("DescriptorType", &"CodeDataSegment")
            .field(
                "DescriptorPrivilegeLevel",
                &self.get_descriptor_privilege_level(),
            )
            .field("Present", &self.contains(Self::PRESENT))
            .finish()
    }
}

bitflags! {
    pub struct SystemSegmentAccessByte: u8 {
        /// The type of descriptor.
        ///
        /// If clear, this is a System Segment.
        /// If set, this is a code or data segment.
        const CODE_DATA_SEGMENT = 1 << 4;

        /// If the segment is present
        const PRESENT = 1 << 7;
    }
}

impl SystemSegmentAccessByte {
    /// Set descriptor privilege level
    pub const fn descriptor_privilege_level(mut self, level: u8) -> Self {
        self.bits |= (level & 0x2) << 6;
        self
    }

    /// Get descriptor privilege level
    pub const fn get_descriptor_privilege_level(&self) -> u8 {
        (self.bits >> 6) & 0x2
    }

    /// Set segment type
    pub const fn segment_type(mut self, dtype: SegmentType) -> Self {
        self.bits |= (unsafe { dtype.untyped } & 0b1_1111);
        self
    }

    /// Get segment type
    pub const fn get_segment_type(&self) -> SegmentType {
        unsafe { core::mem::transmute(self.bits & 0b1_1111) }
    }
}

impl Debug for SystemSegmentAccessByte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SystemSegmentAccessByte")
            .field("SegmentType", &self.get_segment_type())
            .field("DescriptorType", &"SystemSegment")
            .field(
                "DescriptorPrivilegeLevel",
                &self.get_descriptor_privilege_level(),
            )
            .field("Present", &self.contains(Self::PRESENT))
            .finish()
    }
}

bitflags! {
    pub struct GenericAccessByte: u8 {
        /// The type of descriptor.
        ///
        /// If clear, this is a System Segment.
        /// If set, this is a code or data segment.
        const DESCRIPTOR_TYPE = 1 << 4;

        /// If the segment is present
        const PRESENT = 1 << 7;
    }
}

impl GenericAccessByte {
    /// Get the inner `CodeDataSegmentAccessByte` if the access byte is of that type
    pub const fn get_code_or_data(self) -> Option<CodeDataSegmentAccessByte> {
        if self.contains(Self::DESCRIPTOR_TYPE) {
            Some(unsafe { CodeDataSegmentAccessByte::from_bits_unchecked(self.bits) })
        } else {
            None
        }
    }

    /// Get the inner `SystemSegmentAccessByte` if the access byte is of that type
    pub const fn get_system_segment(self) -> Option<SystemSegmentAccessByte> {
        if !self.contains(Self::DESCRIPTOR_TYPE) {
            Some(unsafe { SystemSegmentAccessByte::from_bits_unchecked(self.bits) })
        } else {
            None
        }
    }
}

impl core::fmt::Debug for GenericAccessByte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(cd) = self.get_code_or_data() {
            write!(f, "{:#?}", cd)
        } else if let Some(ss) = self.get_system_segment() {
            write!(f, "{:#?}", ss)
        } else {
            unreachable!()
        }
    }
}

bitflags! {
    pub struct Flags: u8 {
        /// If set, this descriptor defines a 64 bit segment. When set, SIZE should be unset. Any other descriptor value requires this be unset
        const LONG_MODE = 1 << 1;
        /// If clear, this describes a 16 bit segment, else it describes a 32 bit protected segment
        const SIZE = 1 << 2;
        /// If unset, byte granularity is used, else page granularity
        const GRANULARITY = 1 << 3;
    }
}

impl Debug for Flags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Flags")
            .field(
                "Granularity",
                if self.contains(Self::GRANULARITY) {
                    &"Page"
                } else {
                    &"Byte"
                },
            )
            .field(
                "Type",
                &match (self.contains(Self::LONG_MODE), self.contains(Self::SIZE)) {
                    (true, false) => "Long Mode (64-bit)",
                    (false, true) => "Protected Mode (32-bit)",
                    (false, false) => "Real Mode (16-bit)",
                    (true, true) => "INVALID CONFIG",
                },
            )
            .finish()
    }
}

/*
L=0 D=0 for 16 bit
L=0 D=1 for 32 bit
L=1 D=0 for 64 bit
*/

#[repr(C, packed)]
#[derive(Clone, Copy)]
/// A Segment Descriptor
pub struct SegmentDescriptor {
    /// Limit of the descriptor, ignored in 64 bit mode
    pub limit: u16,
    /// Bits 0-15 of the base
    pub base1: u16,
    /// Bits 16-23 of the base
    pub base2: u8,
    /// The Access Byte
    pub access_byte: GenericAccessByte,
    flags_and_limit: u8,
    /// Bits 24-31 of the base
    pub base3: u8,
}

impl SegmentDescriptor {
    /// Create a new zeroed Segment Descriptor
    pub const fn new() -> Self {
        Self {
            limit: 0,
            base1: 0,
            base2: 0,
            access_byte: GenericAccessByte::from_bits_truncate(0),
            flags_and_limit: 0,
            base3: 0,
        }
    }

    /// Get the flags
    pub fn get_flags(&self) -> Flags {
        unsafe { Flags::from_bits_unchecked((self.flags_and_limit >> 4) & 0xF) }
    }

    /// Set the flags
    pub const fn flags(mut self, flags: Flags) -> Self {
        self.flags_and_limit |= flags.bits() << 4;
        self
    }

    /// Get the limit
    pub fn get_limit(&self) -> u32 {
        (self.flags_and_limit as u32) << 4 | self.limit as u32
    }

    /// Set the limit
    pub const fn limit(mut self, val: u32) -> Self {
        let l1u16: u16 = (val & 0xFFFF) as u16;
        let l1u8: u8 = ((val >> 4) & 0xF) as u8;
        self.limit = l1u16;
        self.flags_and_limit |= l1u8 & 0xF;
        self
    }

    /// Get the base in the segment
    pub fn get_base(&self) -> u32 {
        let b1: u32 = self.base1.into();
        let b2: u32 = (self.base2 as u32) << 16;
        let b3: u32 = (self.base3 as u32) << 24;
        b1 | b2 | b3
    }

    /// Set the base in the segment
    pub const fn base(mut self, base: u32) -> Self {
        let b1: u16 = (base & 0xFFFF) as u16;
        let b2: u8 = ((base >> 16) & 0xFF) as u8;
        let b3: u8 = ((base >> 24) & 0xFF) as u8;
        self.base1 = b1;
        self.base2 = b2;
        self.base3 = b3;
        self
    }
}

impl core::fmt::Debug for SegmentDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if unsafe { core::mem::transmute::<SegmentDescriptor, u64>(*self) == 0 } {
            f.debug_struct("SegmentDescriptor")
                .field("Null", &true)
                .finish()
        } else {
            f.debug_struct("SegmentDescriptor")
                .field("Limit", &format_args!("{:#X}", self.get_limit()))
                .field("Base", &format_args!("{:#X}", self.get_base()))
                .field("AccessByte", &self.access_byte)
                .field("Flags", &self.get_flags())
                .finish_non_exhaustive()
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
/// An entry in the GDT
pub struct SystemSegmentDescriptorLongMode {
    /// The maximum addressable unit and the size of the segment, ignored in 64bit
    pub limit: u16,
    /// Bytes 0-3 of the base
    pub base1: u16,
    /// Bytes 4-7 of the base
    pub base2: u8,
    /// The access byte, determining what the segment is used for
    pub access_byte: SystemSegmentAccessByte,
    /// Limit is the lower four bits, flags is the upper four bits
    pub flags_and_limit: u8,
    /// Bytes 8-11 of the base
    pub base3: u8,
    /// Bytes 12-15 of the base
    pub base4: u32,
    reserved: u32,
}

impl SystemSegmentDescriptorLongMode {
    /// Create a completely zeroed entry, for later modification.
    pub const fn new_unused() -> Self {
        Self {
            limit: 0,
            base1: 0,
            base2: 0,
            access_byte: SystemSegmentAccessByte::from_bits_truncate(0),
            flags_and_limit: 0,
            base3: 0,
            base4: 0,
            reserved: 0,
        }
    }

    /// Get the flags
    ///
    /// `Flag Values`
    /// * G: Granularity flag, indicates the size the Limit value is scaled by. If clear (0), the Limit is in 1 Byte blocks (byte granularity). If set (1), the Limit is in 4 KiB blocks (page granularity).
    /// * DB: Size flag. If clear (0), the descriptor defines a 16-bit protected mode segment. If set (1) it defines a 32-bit protected mode segment. A GDT can have both 16-bit and 32-bit selectors at once.
    /// * L: Long-mode code flag. If set (1), the descriptor defines a 64-bit code segment. When set, Sz should always be clear. For any other type of segment (other code types or any data segment), it should be clear (0).
    pub fn get_flags(&self) -> u8 {
        (self.flags_and_limit >> 4) & 0xF
    }

    /// Set the flags
    /// `Flag Values`
    /// * G: Granularity flag, indicates the size the Limit value is scaled by. If clear (0), the Limit is in 1 Byte blocks (byte granularity). If set (1), the Limit is in 4 KiB blocks (page granularity).
    /// * DB: Size flag. If clear (0), the descriptor defines a 16-bit protected mode segment. If set (1) it defines a 32-bit protected mode segment. A GDT can have both 16-bit and 32-bit selectors at once.
    /// * L: Long-mode code flag. If set (1), the descriptor defines a 64-bit code segment. When set, Sz should always be clear. For any other type of segment (other code types or any data segment), it should be clear (0).
    pub fn set_flags(&mut self, val: u8) {
        self.flags_and_limit |= val << 4;
    }

    /// Get the limit
    pub fn get_limit(&self) -> u32 {
        (self.flags_and_limit as u32 & 0xF) << 16 | self.limit as u32
    }

    /// Set the limit
    pub fn set_limit(&mut self, val: u32) {
        let l1u16: u16 = (val & 0xFFFF).try_into().unwrap();
        let l1u8: u8 = ((val >> 4) & 0xF).try_into().unwrap();
        self.limit = l1u16;
        self.flags_and_limit |= l1u8 & 0xF;
    }

    /// Get the base in the segment
    pub fn get_base(&self) -> u64 {
        let b1: u64 = self.base1.into();
        let b2: u64 = self.base2.into();
        let b3: u64 = self.base3.into();
        let b4: u64 = self.base4.into();
        b1 | b2 | b3 | b4
    }

    /// Set the base in the segment
    pub fn set_base(&mut self, base: u64) {
        let b1: u16 = (base & 0xFFFF).try_into().unwrap();
        let b2: u8 = ((base << 16) & 0xFF).try_into().unwrap();
        let b3: u8 = ((base << 24) & 0xFF).try_into().unwrap();
        let b4: u32 = (base << 32) as u32;
        self.base1 = b1;
        self.base2 = b2;
        self.base3 = b3;
        self.base4 = b4;
    }
}

impl core::fmt::Debug for SystemSegmentDescriptorLongMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SystemSegmentDescriptor")
            .field("limit", &format_args!("{:#X}", self.get_limit()))
            .field("base", &format_args!("{:#X}", self.get_base()))
            .field("access_byte", &self.access_byte)
            .field("flags", &self.get_flags())
            .finish_non_exhaustive()
    }
}

/// Results from SGDT
#[repr(packed, C)]
pub struct SizedDescriptorTable {
    /// The limit of the GDT
    pub limit: u16,
    /// The base address of the GDT
    pub base: u64,
}

impl SizedDescriptorTable {
    /// Get the current GDT
    pub fn get_gdt() -> Self {
        let mut tmp = SizedDescriptorTable { limit: 0, base: 0 };
        let ptr = &mut tmp as *mut SizedDescriptorTable;
        unsafe {
            asm!("sgdt [{}]", in(reg) ptr);
        }
        tmp
    }
}

impl Debug for SizedDescriptorTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SizedDescriptorTable")
            .field("limit", &unsafe {
                (self as *const Self as *const u16).read_unaligned()
            })
            .field(
                "base",
                &format_args!("{:#?}", unsafe {
                    ((&self as *const _ as *const u16).offset(1) as *const u64).read_unaligned()
                        as *const u8
                }),
            )
            .finish()
    }
}

#[derive(Debug, Default)]
#[repr(C)]
/// A representation of the Global Descriptor Table
pub struct GlobalDescriptorTable<'a> {
    /// The entries of the GDT
    pub entries: &'a mut [SegmentDescriptor],
}

impl<'a> GlobalDescriptorTable<'a> {
    /// The kernel code segment
    pub const KCODE: u16 = 1 << 3;

    /// The kernel data segment
    pub const KDATA: u16 = 2 << 3;

    /// The 32 bit user code segment
    pub const UCODE32: u16 = 3 << 3;

    /// The 32 bit user data segment
    pub const UDATA32: u16 = 4 << 3;

    /// The 64 bit user code segment
    pub const UCODE64: u16 = 5 << 3;

    /// The 64 bit user data segment
    pub const UDATA64: u16 = 6 << 3;

    /// Create a global descriptor table from an exist SizedDescriptorTable
    pub fn from_existing(res: SizedDescriptorTable) -> Self {
        let limit: usize = ((res.limit + 1) / 8).into();
        Self {
            entries: unsafe {
                core::slice::from_raw_parts_mut(res.base as *mut SegmentDescriptor, limit)
            },
        }
    }

    /// Apply the changes
    #[naked]
    pub extern "sysv64" fn apply(from: usize) {
        const KCODE: u16 = GlobalDescriptorTable::KCODE;
        const KDATA: u16 = GlobalDescriptorTable::KDATA;
        unsafe {
            asm!(
                "lgdt [rdi]",
                "mov   AX, {0}",
                "mov   DS, AX",
                "mov   ES, AX",
                "mov   FS, AX",
                "mov   GS, AX",
                "mov   SS, AX",
                "pop rax",
                "push {1}",
                "push rax",
                "retfq",
                const KDATA,
                const KCODE,
                options(noreturn)
            )
        }
    }
}

impl<'a> Index<usize> for GlobalDescriptorTable<'a> {
    type Output = SegmentDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<'a> IndexMut<usize> for GlobalDescriptorTable<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}
