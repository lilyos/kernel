use core::{
    arch::asm,
    fmt::Debug,
    ops::{Index, IndexMut},
};

/// Entry types for the GDT in long mode
#[repr(u8)]
pub enum EntryType {
    /// If the entry is a LDT
    Ldt = 0x2,
    /// If the entry is a TSS
    Tss = 0x9,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
/// The access byte for a Task State Segment or a Local Descriptor Table
pub struct TSSAccessByte(pub u8);

impl TSSAccessByte {
    /// Gets the Present bit (8).
    /// Marks a segment as being a valid entry in the GDT.
    pub fn get_present(&self) -> bool {
        self.0 & (1 << 7) != 0
    }

    /// Sets the Present bit (8).
    /// Marks a segment as being a valid entry in the GDT.
    pub fn set_present(&mut self) {
        self.0 |= 1 << 7
    }

    /// Clears the Present bit (8).
    /// Marks a segment as being a valid entry in the GDT.
    pub fn clear_present(&mut self) {
        self.0 &= !(1 << 7);
    }

    /// Gets the Descriptor Level (bits 6-7).
    /// Describes the privilege level of the segment.
    pub fn get_descriptor_level(&self) -> u8 {
        (self.0 >> 5) & 0b11
    }

    /// Sets the Descriptor Level (bits 6-7).
    /// Describes the privilege level of the segment.
    pub fn set_descriptor_level(&mut self, level: u8) {
        self.0 &= 0b1001_1111;
        self.0 &= (level << 5) & 0b0110_0000;
    }

    /// Get the type of the segment
    pub fn get_type(&self) -> u8 {
        self.0 & 0xF
    }

    /// Set the type of the segment
    pub fn set_type(&mut self, kind: EntryType) {
        self.0 &= !0xF;
        self.0 &= kind as u8;
    }
}

impl core::fmt::Debug for TSSAccessByte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TSSAccessByte")
            .field("type", &format_args!("0x{:x}", self.get_type()))
            .field("descriptor_level", &self.get_descriptor_level())
            .field("present", &self.get_present())
            .finish()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
/// The access byte for a code or data segment
pub struct CodeDataAccessByte(pub u8);

impl CodeDataAccessByte {
    /// Gets the Present bit (8).
    /// Marks a segment as being a valid entry in the GDT.
    pub fn get_present(&self) -> bool {
        self.0 & (1 << 7) != 0
    }

    /// Sets the Present bit (8).
    /// Marks a segment as being a valid entry in the GDT.
    pub fn set_present(&mut self) {
        self.0 |= 1 << 7
    }

    /// Clears the Present bit (8).
    /// Marks a segment as being a valid entry in the GDT.
    pub fn clear_present(&mut self) {
        self.0 &= !(1 << 7);
    }

    /// Gets the Descriptor Level (bits 6-7).
    /// Describes the privilege level of the segment.
    pub fn get_descriptor_level(&self) -> u8 {
        (self.0 >> 5) & 0b11
    }

    /// Sets the Descriptor Level (bits 6-7).
    /// Describes the privilege level of the segment.
    pub fn set_descriptor_level(&mut self, level: u8) {
        self.0 &= 0b1001_1111;
        self.0 &= (level << 5) & 0b011;
    }

    /// Gets the Executable Bit (4).
    /// Determines if the descriptor defines a data segment (0) or a code segment (1).
    pub fn get_executable(&self) -> bool {
        self.0 & (1 << 3) != 0
    }

    /// Sets the Executable Bit (4).
    /// Determines if the descriptor defines a data segment (0) or a code segment (1).
    pub fn set_executable(&mut self) {
        self.0 |= 1 << 3;
    }

    /// Clears the Executable Bit (4).
    /// Determines if the descriptor defines a data segment (0) or a code segment (1).
    pub fn clear_executable(&mut self) {
        self.0 &= !(1 << 3)
    }

    /// Gets the Direction Bit/Conforming Bit (3).
    /// For data selectors, it determines if the segment grows up (0) or down (1).
    /// For code selectors, it determines if the code can only be executed from the current privilege level (0) or from an equal or lower privilege level (1).
    pub fn get_direction(&self) -> bool {
        self.0 & (1 << 2) != 0
    }

    /// Sets the Direction Bit/Conforming Bit (3).
    /// For data selectors, it determines if the segment grows up (0) or down (1).
    /// For code selectors, it determines if the code can only be executed from the current privilege level (0) or from an equal or lower privilege level (1).
    pub fn set_direction(&mut self) {
        self.0 |= 1 << 2
    }

    /// Clears the Direction Bit/Conforming Bit (3).
    /// For data selectors, it determines if the segment grows up (0) or down (1).
    /// For code selectors, it determines if the code can only be executed from the current privilege level (0) or from an equal or lower privilege level (1).
    pub fn clear_direction(&mut self) {
        self.0 &= !(1 << 2)
    }

    /// Gets the Read-Write bit (2).
    /// For code segments, it is the readable bit. If clear (0), reading is not allowed. Writing is never allowed for code segments.
    /// For data segments, it is the writable bit. If clear(0), writing is not allowed. Reading is always allowed for data segments.
    pub fn get_read_write(&self) -> bool {
        self.0 & 1 << 1 != 0
    }

    /// Sets the Read-Write bit (2).
    /// For code segments, it is the readable bit. If clear (0), reading is not allowed. Writing is never allowed for code segments.
    /// For data segments, it is the writable bit. If clear(0), writing is not allowed. Reading is always allowed for data segments.
    pub fn set_read_write(&mut self) {
        self.0 |= 1 << 1
    }

    /// Clears the Read-Write bit (2).
    /// For code segments, it is the readable bit. If clear (0), reading is not allowed. Writing is never allowed for code segments.
    /// For data segments, it is the writable bit. If clear(0), writing is not allowed. Reading is always allowed for data segments.
    pub fn clear_read_write(&mut self) {
        self.0 &= !(1 << 1)
    }

    /// Gets the Accessed bit (1).
    /// If clear (0), a segment has not been accessed. It will be set automatically by the cpu.
    pub fn get_accessed(&self) -> bool {
        self.0 & (1 << 0) != 0
    }

    /// Sets the Accessed bit (1).
    /// If clear (0), a segment has not been accessed. It will be set automatically by the cpu.
    pub fn set_accessed(&mut self) {
        self.0 |= 1 << 0
    }

    /// Clears the Accessed bit (1).
    /// If clear (0), a segment has not been accessed. It will be set automatically by the cpu.
    pub fn clear_accessed(&mut self) {
        self.0 &= !(1 << 0);
    }
}

impl core::fmt::Debug for CodeDataAccessByte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CodeDataAccessByte")
            .field("accessed", &self.get_accessed())
            .field("read_write", &self.get_read_write())
            .field("direction", &self.get_direction())
            .field("executable", &self.get_executable())
            .field("descriptor_level", &self.get_descriptor_level())
            .field("present", &self.get_present())
            .finish()
    }
}

#[derive(Clone, Copy)]
/// An undetermined type of access byte
pub union AccessByte {
    /// If it's for a TSS
    pub tss: TSSAccessByte,
    /// If it's for a code or data segment
    pub code: CodeDataAccessByte,
    /// The raw value
    raw: u8,
}

impl AccessByte {
    /// Get if the segment is present
    pub fn get_present(&self) -> bool {
        unsafe { self.raw & 0b100_0000 != 0 }
    }

    /// Set if the segment is present
    pub fn set_present(&mut self, val: bool) {
        unsafe { self.raw ^= val as u8 & (1 << 7) }
    }

    /// Get if the segment is for a task state segment
    pub fn is_tss(&self) -> bool {
        unsafe { (self.raw >> 5) & 0b1 == 0 }
    }

    /// Get if the segment is for a code or data segment
    pub fn is_code_or_data(&self) -> bool {
        unsafe { (self.raw >> 5) & 0b1 == 1 }
    }

    /// Set the segment type
    pub fn set_is_code_or_data(&mut self, val: bool) {
        unsafe {
            self.raw |= ((val as u8) & 0b1) << 5;
        }
    }
}

impl core::fmt::Debug for AccessByte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_code_or_data() {
            write!(f, "{:#?}", unsafe { self.code })
        } else {
            write!(f, "{:#?}", unsafe { self.tss })
        }
    }
}

/*
L=0 D=0 for 16 bit
L=0 D=1 for 32 bit
L=1 D=0 for 64 bit
*/
#[repr(C, packed)]
#[derive(Clone, Copy)]
/// An entry in the GDT
pub struct SegmentDescriptor {
    /// The maximum addressable unit and the size of the segment, ignored in 64bit
    pub limit: u16,
    /// Bytes 0-3 of the base
    pub base1: u16,
    /// Bytes 4-7 of the base
    pub base2: u8,
    /// The access byte, determining what the segment is used for
    pub access_byte: AccessByte,
    /// Limit is the lower four bits, flags is the upper four bits
    flags_and_limit: u8,
    /// Bytes 8-11 of the base
    pub base3: u8,
    /// Bytes 12-15 of the base
    pub base4: u32,
    reserved: u32,
}

impl SegmentDescriptor {
    /// Create a completely zeroed entry, for later modification.
    pub const fn new_unused() -> Self {
        Self {
            limit: 0,
            base1: 0,
            base2: 0,
            access_byte: AccessByte { raw: 0 },
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
        let b4: u32 = (base << 32).try_into().unwrap();
        self.base1 = b1;
        self.base2 = b2;
        self.base3 = b3;
        self.base4 = b4;
    }
}

impl core::fmt::Debug for SegmentDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SegmentDescriptor")
            .field("limit", &format_args!("0x{:x}", self.get_limit()))
            .field("base", &format_args!("0x{:x}", self.get_base()))
            .field("access_byte", &self.access_byte)
            .field("flags", &format_args!("0x{:x}", self.get_flags()))
            .finish_non_exhaustive()
    }
}

/// Results from SGDT
#[repr(packed, C)]
pub struct SaveGlobalDescriptorTableResult {
    /// The limit of the GDT
    pub limit: u16,
    /// The base address of the GDT
    pub base: u64,
}

impl SaveGlobalDescriptorTableResult {
    /// Get the current GDT
    pub fn get() -> Self {
        let mut tmp = SaveGlobalDescriptorTableResult { limit: 0, base: 0 };
        let ptr = &mut tmp as *mut SaveGlobalDescriptorTableResult;
        unsafe {
            asm!("sgdt [{}]", in(reg) ptr);
        }
        tmp
    }
}

impl Debug for SaveGlobalDescriptorTableResult {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SaveGlobalDescriptorTableResult")
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
    /// Create a global descriptor table from an exist SaveGlobalDescriptorTableResult
    pub fn from_existing(res: SaveGlobalDescriptorTableResult) -> Self {
        let limit: usize = ((res.limit + 1) / 8).into();
        Self {
            entries: unsafe {
                core::slice::from_raw_parts_mut(res.base as *mut SegmentDescriptor, limit)
            },
        }
    }

    /// Apply the changes
    const KCODE: u16 = 0b0000_0000_0000_1000;
    const KDATA: u16 = 0b0000_0000_0001_0000;
    #[naked]
    pub extern "sysv64" fn apply(from: SaveGlobalDescriptorTableResult) {
        asm!(
            "lgdt [rdi]",

            "mov   AX, {1}",
            "mov   DS, AX",
            "mov   ES, AX",
            "mov   FS, AX",
            "mov   GS, AX",
            "mov   SS, AX",

            "pop rax",
            "push word ptr {2}",
            "push rax",
            "retfq",
            inout("rdi") _,
            out("rax") _,
            const KDATA,
            const KCODE,
            options(noreturn)
        )
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
