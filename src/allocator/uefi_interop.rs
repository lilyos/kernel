use super::align;

#[allow(non_camel_case_types, dead_code)]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryType {
    /// This enum variant is not used.
    RESERVED = 0,
    /// The code portions of a loaded UEFI application.
    LOADER_CODE = 1,
    /// The data portions of a loaded UEFI applications,
    /// as well as any memory allocated by it.
    LOADER_DATA = 2,
    /// Code of the boot drivers.
    ///
    /// Can be reused after OS is loaded.
    BOOT_SERVICES_CODE = 3,
    /// Memory used to store boot drivers' data.
    ///
    /// Can be reused after OS is loaded.
    BOOT_SERVICES_DATA = 4,
    /// Runtime drivers' code.
    RUNTIME_SERVICES_CODE = 5,
    /// Runtime services' code.
    RUNTIME_SERVICES_DATA = 6,
    /// Free usable memory.
    CONVENTIONAL = 7,
    /// Memory in which errors have been detected.
    UNUSABLE = 8,
    /// Memory that holds ACPI tables.
    /// Can be reclaimed after they are parsed.
    ACPI_RECLAIM = 9,
    /// Firmware-reserved addresses.
    ACPI_NON_VOLATILE = 10,
    /// A region used for memory-mapped I/O.
    MMIO = 11,
    /// Address space used for memory-mapped port I/O.
    MMIO_PORT_SPACE = 12,
    /// Address space which is part of the processor.
    PAL_CODE = 13,
    /// Memory region which is usable and is also non-volatile.
    PERSISTENT_MEMORY = 14,
}

/// A structure describing a region of memory.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryDescriptor {
    /// Type of memory occupying this range.
    pub ty: MemoryType,
    /// Skip 4 bytes as UEFI declares items in structs should be naturally aligned
    padding: u32,
    /// Starting physical address.
    pub phys_start: u64,
    /// Starting virtual address.
    pub virt_start: u64,
    /// Number of 4 KiB pages contained in this range.
    pub page_count: u64,
    /// The capability attributes of this memory range.
    att: u64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MemoryKind {
    Reserve,
    Reclaim,
    ACPIReclaim,
    ACPINonVolatile,
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryEntry {
    pub start: usize,
    pub end: usize,
    pub kind: MemoryKind,
}

impl From<&MemoryDescriptor> for MemoryEntry {
    fn from(memd: &MemoryDescriptor) -> Self {
        MemoryEntry {
            start: align(memd.phys_start as usize, 4096),
            end: align(
                (memd.phys_start + memd.page_count * 4096 - 1) as usize,
                4096,
            ),
            kind: if memd.ty == MemoryType::BOOT_SERVICES_CODE
                || memd.ty == MemoryType::BOOT_SERVICES_DATA
                || memd.ty == MemoryType::CONVENTIONAL
            {
                MemoryKind::Reclaim
            } else if memd.ty == MemoryType::ACPI_RECLAIM {
                MemoryKind::ACPIReclaim
            } else if memd.ty == MemoryType::ACPI_NON_VOLATILE {
                MemoryKind::ACPINonVolatile
            } else {
                MemoryKind::Reserve
            },
        }
    }
}
