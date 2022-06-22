use core::mem;

use crate::interrupts::InterruptType;

use super::SizedDescriptorTable;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
/// Type attributes for an IDT entry
pub struct InterruptDescriptorTypeAttributes(pub u8);

impl InterruptDescriptorTypeAttributes {
    /// If the interrupt is for an interrupt
    const INTERRUPT_GATE: u8 = 0xE;
    /// If the interrupt is for a trap
    #[allow(dead_code)]
    const TRAP_GATE: u8 = 0xF;

    /// Create a new empty descriptor
    pub const fn new() -> Self {
        Self(0)
    }

    /// Create a new interrupt descriptor attribute set
    pub const fn new_interrupt() -> Self {
        Self(Self::INTERRUPT_GATE)
    }

    /// Create a new trap interrupt descriptor attribute set
    pub const fn new_trap() -> Self {
        Self(Self::TRAP_GATE)
    }

    /// Set the descriptor present
    pub const fn set_present(self) -> Self {
        Self(self.0 | (1 << 7))
    }

    /// Set the descriptor privilege level
    pub const fn set_privilege_level(self, level: u8) -> Self {
        Self(self.0 | ((level & 0b11) << 5))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
/// Interrupt descriptor, entry in Interrupt Descriptor Table
pub struct InterruptDescriptor {
    /// Offset bits 0-15
    pub offset_1: u16,
    /// Selector into Global Descriptor Table (GDT) or Local Descriptor Table (LDT)
    pub selector: u16,
    /// Interrupt stack table offset, 0-2 used, rest are zeroed
    pub ist: u8,
    /// Gate type, DPL, and P fields
    pub type_attributes: InterruptDescriptorTypeAttributes,
    /// Offset bits 16-31
    pub offset_2: u16,
    /// Offset bits 32-63
    pub offset_3: u32,
    /// Reserved
    reserved: u32,
}

impl InterruptDescriptor {
    /// Create a zeroed IDT
    pub const fn new_zeroed() -> Self {
        Self {
            offset_1: 0,
            selector: 0,
            ist: 0,
            type_attributes: InterruptDescriptorTypeAttributes::new(),
            offset_2: 0,
            offset_3: 0,
            reserved: 0,
        }
    }

    /// Create a new IDT for a trap
    pub const fn new_trap() -> Self {
        Self {
            offset_1: 0,
            selector: 0,
            ist: 0,
            type_attributes: InterruptDescriptorTypeAttributes::new_trap(),
            offset_2: 0,
            offset_3: 0,
            reserved: 0,
        }
    }

    /// Create a new IDT for a trap
    pub const fn new_interrupt() -> Self {
        Self {
            offset_1: 0,
            selector: 0,
            ist: 0,
            type_attributes: InterruptDescriptorTypeAttributes::new_interrupt(),
            offset_2: 0,
            offset_3: 0,
            reserved: 0,
        }
    }

    /// Set the ISR address
    ///
    /// # Arguments
    /// * `handler` - The ISR handler to set the address to
    pub fn set_isr_address(
        self,
        handler: extern "x86-interrupt" fn(&mut ExceptionStackFrame),
    ) -> Self {
        let addr = handler as usize;
        let p1 = addr as u16;
        let p2 = (addr >> 16) as u16;
        let p3 = (addr >> 32) as u32;
        Self {
            offset_1: p1,
            offset_2: p2,
            offset_3: p3,
            ..self
        }
    }

    /// Set the ISR address
    ///
    /// # Arguments
    /// * `handler` - The ISR handler to set the address to
    pub fn set_isr_address_code(
        self,
        handler: extern "x86-interrupt" fn(&mut ExceptionStackFrame, u64),
    ) -> Self {
        let addr = handler as usize;
        let p1 = addr as u16;
        let p2 = (addr >> 16) as u16;
        let p3 = (addr >> 32) as u32;
        Self {
            offset_1: p1,
            offset_2: p2,
            offset_3: p3,
            ..self
        }
    }

    /// Set the descriptor's type attributes
    pub fn set_type_attributes(self, attributes: InterruptDescriptorTypeAttributes) -> Self {
        Self {
            type_attributes: attributes,
            ..self
        }
    }

    /// Set the selector
    pub fn set_segment(self, selector: u16) -> Self {
        Self { selector, ..self }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
/// The Interrupt Descriptor Table. It holds 256 entries, from 0-255
pub struct InterruptDescriptorTable {
    /// The values
    pub inner: [InterruptDescriptor; 256],
}

impl InterruptDescriptorTable {
    /// Creates a new, empty IDT
    pub const fn new() -> Self {
        Self {
            inner: [InterruptDescriptor::new_zeroed(); 256],
        }
    }

    /// load the IDT
    pub fn load(&self) {
        let tmp = SizedDescriptorTable {
            limit: (self.inner.len() * mem::size_of::<InterruptDescriptor>() - 1) as u16,
            base: self.inner.as_ptr() as u64,
        };
        unsafe { asm!("lidt [{}]", in(reg) &tmp as *const _) }
    }
}

impl Default for InterruptDescriptorTable {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
/// x86-64 Exception Stack Frame
pub struct ExceptionStackFrame {
    /// Address to be returned to after the exception is handled,
    /// usually the instruction after the faulting instruction,
    /// except for Page Faults, in which it restarts the instruction
    pub instruction_pointer: *mut u8,
    /// The code segment selector
    pub code_segment: u64,
    /// The flags register before the interrupt was invoked
    pub cpu_flags: u64,
    /// The stack pointer at the time of the interrupt
    pub stack_pointer: *mut u8,
    /// The stack segment of the descriptor at the time of the interrupt
    pub stack_segment: u64,
}

/// Function signature for handling interrupts for this platform
pub type InterruptHandler = unsafe extern "x86-interrupt" fn(&mut ExceptionStackFrame);

/// The kernel's IDT
#[used]
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

/// The generic kernel interrupt handler
#[used]
pub static mut INTERRUPT_HANDLER: Option<fn(InterruptType)> = None;

/// Install the interrupt handler
/// # Safety
/// This may only be called from one core, otherwise read/write tearing may occur
#[allow(dead_code)]
pub unsafe fn install_interrupt_handler() {
    // DivideByZero
    /// Divide by zero
    const DIVIDE_BY_ZERO: usize = 0;

    // DebugBreakpoint
    /// Debug Exception
    const DEBUG: usize = 1;
    /// Breakpoint
    const BREAKPOINT: usize = 3;

    // Generic
    /// Caused for a variety of reasons
    const GENERAL_PROTECTION_FAULT: usize = 13;

    /// INTO instruction used while overflow bit is set
    const OVERFLOW: usize = 4;

    /// BOUND instruction exceeded range
    const BOUND_RANGE_EXCEEDED: usize = 5;

    // InvalidInstruction
    /// Invalid Opcode
    const INVALID_OPCODE: usize = 6;

    // InvalidAccess
    /// Page fault, address not mapped or accessor unprivileged
    const PAGE_FAULT: usize = 14;

    // CheckFailed
    /// Unaligned access
    const ALIGNMENT_CHECK: usize = 17;
    /// An internal error occurred in the processor, ram, or so forth
    const MACHINE_CHECK: usize = 18;

    /// A device isn't available
    const DEVICE_NOT_AVAILABLE: usize = 7;

    // #9 Coprocessor Segment Overrun - No longer recommended
    const INVALID_TSS: usize = 10;
    const SEGMENT_NOT_PRESENT: usize = 11;
    const STACK_SEGMENT_FAULT: usize = 12;

    // SIMDError
    const SIMD_FLOATING_POINT: usize = 19;

    // FloatingPoint
    const FLOATING_POINT: usize = 16;

    // VirtualizationError
    /// Virtualization exception
    const VIRTUALIZATION: usize = 20;
    /// The host is trying to communicate with the guest
    const VMM_COMMUNICATION: usize = 29;

    // HypervisorInterference
    /// The Hypervisor is attempting to modify the configuration of the guest
    const HYPERVISOR_INJECTION: usize = 28;

    // ControlprotectionViolation
    /// Processor control mechanism was violated
    const CONTROL_PROTECTION: usize = 21;
    /// Processor security mechanism was violated
    const SECURITY_VIOLATION: usize = 30;

    // NonMaskableInterrupt
    /// Non maskable interrupt
    const NMI: usize = 2;
    /// A fault handler faulted
    const DOUBLE_FAULT: usize = 8;

    // #15 Reserved
    // #22-27 Reserved

    let idt = &mut IDT.inner;

    macro_rules! create_interrupt {
        ($idx:expr, $handler:ident, $segment:expr, $priv_level:expr) => {
            idt[$idx] = InterruptDescriptor::new_interrupt()
                .set_type_attributes(
                    InterruptDescriptorTypeAttributes::new_interrupt()
                        .set_present()
                        .set_privilege_level($priv_level),
                )
                .set_isr_address($handler)
                .set_segment($segment);
        };
    }

    macro_rules! create_interrupt_code {
        ($idx:expr, $handler:ident, $segment:expr, $priv_level:expr) => {
            idt[$idx] = InterruptDescriptor::new_interrupt()
                .set_type_attributes(
                    InterruptDescriptorTypeAttributes::new_interrupt()
                        .set_present()
                        .set_privilege_level($priv_level),
                )
                .set_isr_address_code($handler)
                .set_segment($segment);
        };
    }

    use super::handlers::*;
    use super::GlobalDescriptorTable as GDT;

    create_interrupt!(DIVIDE_BY_ZERO, divide_by_zero, GDT::KCODE, 0);
    create_interrupt!(DEBUG, debug, GDT::KCODE, 0);
    create_interrupt!(BREAKPOINT, breakpoint, GDT::KCODE, 0);
    create_interrupt_code!(GENERAL_PROTECTION_FAULT, general_protection, GDT::KCODE, 0);
    create_interrupt!(OVERFLOW, overflow, GDT::KCODE, 0);
    create_interrupt!(BOUND_RANGE_EXCEEDED, bound_range_exceeded, GDT::KCODE, 0);
    create_interrupt!(INVALID_OPCODE, invalid_opcode, GDT::KCODE, 0);
    create_interrupt_code!(PAGE_FAULT, page_fault, GDT::KCODE, 0);
    create_interrupt_code!(ALIGNMENT_CHECK, alignment, GDT::KCODE, 0);
    create_interrupt!(MACHINE_CHECK, machine, GDT::KCODE, 0);
    create_interrupt!(DEVICE_NOT_AVAILABLE, device_not_available, GDT::KCODE, 0);
    create_interrupt_code!(INVALID_TSS, invalid_tss, GDT::KCODE, 0);
    create_interrupt_code!(SEGMENT_NOT_PRESENT, segment_not_present, GDT::KCODE, 0);
    create_interrupt_code!(STACK_SEGMENT_FAULT, stack_segment_fault, GDT::KCODE, 0);
    create_interrupt!(SIMD_FLOATING_POINT, simd_floating_point, GDT::KCODE, 0);
    create_interrupt!(FLOATING_POINT, floating_point, GDT::KCODE, 0);
    create_interrupt!(VIRTUALIZATION, virtualization, GDT::KCODE, 0);
    create_interrupt_code!(VMM_COMMUNICATION, vmm_communication, GDT::KCODE, 0);
    create_interrupt!(HYPERVISOR_INJECTION, hypervisor_injection, GDT::KCODE, 0);
    create_interrupt_code!(CONTROL_PROTECTION, control_protection, GDT::KCODE, 0);
    create_interrupt_code!(SECURITY_VIOLATION, security_violation, GDT::KCODE, 0);
    create_interrupt!(NMI, nmi, GDT::KCODE, 0);
    create_interrupt_code!(DOUBLE_FAULT, double_fault, GDT::KCODE, 0);

    IDT.load();
}
