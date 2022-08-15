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

/// The generic kernel interrupt handler
#[used]
pub static mut INTERRUPT_HANDLER: Option<fn(InterruptType)> = None;

/// Install the interrupt handler
/// # Safety
/// This may only be called from one core, otherwise read/write tearing may occur
#[allow(dead_code)]
pub unsafe fn install_interrupt_handler() {
    use super::handlers::*;
    use super::GlobalDescriptorTable as GDT;

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

    use crate::traits::Platform;

    let idt_o = crate::arch::PLATFORM_MANAGER.get_interrupt_manager().idt();

    let mut idt = { idt_o.inner };

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

    macro_rules! create_generic_hook {
        ($idx:expr, $segment:expr, $priv_level:expr) => {{
            extern "x86-interrupt" fn handle_generic(frame: &mut ExceptionStackFrame) {
                unsafe {
                    INTERRUPT_HANDLER.expect("INTERRUPT HANDLER NOT INSTALLED")(
                        InterruptType::Generic(crate::interrupts::GenericContext {
                            pid: 0,
                            iptr: frame.instruction_pointer.try_into().unwrap(),
                            interrupt_number: $idx,
                            error_code: None,
                        }),
                    )
                }
            }
            idt[$idx] = InterruptDescriptor::new_interrupt()
                .set_type_attributes(
                    InterruptDescriptorTypeAttributes::new_interrupt()
                        .set_present()
                        .set_privilege_level($priv_level),
                )
                .set_isr_address(handle_generic)
                .set_segment($segment);
        }};
    }

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

    create_generic_hook!(32, GDT::KCODE, 0);
    create_generic_hook!(33, GDT::KCODE, 0);
    create_generic_hook!(34, GDT::KCODE, 0);
    create_generic_hook!(35, GDT::KCODE, 0);
    create_generic_hook!(36, GDT::KCODE, 0);
    create_generic_hook!(37, GDT::KCODE, 0);
    create_generic_hook!(38, GDT::KCODE, 0);
    create_generic_hook!(39, GDT::KCODE, 0);
    create_generic_hook!(40, GDT::KCODE, 0);
    create_generic_hook!(41, GDT::KCODE, 0);
    create_generic_hook!(42, GDT::KCODE, 0);
    create_generic_hook!(43, GDT::KCODE, 0);
    create_generic_hook!(44, GDT::KCODE, 0);
    create_generic_hook!(45, GDT::KCODE, 0);
    create_generic_hook!(46, GDT::KCODE, 0);
    create_generic_hook!(47, GDT::KCODE, 0);
    create_generic_hook!(48, GDT::KCODE, 0);
    create_generic_hook!(49, GDT::KCODE, 0);
    create_generic_hook!(50, GDT::KCODE, 0);
    create_generic_hook!(51, GDT::KCODE, 0);
    create_generic_hook!(52, GDT::KCODE, 0);
    create_generic_hook!(53, GDT::KCODE, 0);
    create_generic_hook!(54, GDT::KCODE, 0);
    create_generic_hook!(55, GDT::KCODE, 0);
    create_generic_hook!(56, GDT::KCODE, 0);
    create_generic_hook!(57, GDT::KCODE, 0);
    create_generic_hook!(58, GDT::KCODE, 0);
    create_generic_hook!(59, GDT::KCODE, 0);
    create_generic_hook!(60, GDT::KCODE, 0);
    create_generic_hook!(61, GDT::KCODE, 0);
    create_generic_hook!(62, GDT::KCODE, 0);
    create_generic_hook!(63, GDT::KCODE, 0);
    create_generic_hook!(64, GDT::KCODE, 0);
    create_generic_hook!(65, GDT::KCODE, 0);
    create_generic_hook!(66, GDT::KCODE, 0);
    create_generic_hook!(67, GDT::KCODE, 0);
    create_generic_hook!(68, GDT::KCODE, 0);
    create_generic_hook!(69, GDT::KCODE, 0);
    create_generic_hook!(70, GDT::KCODE, 0);
    create_generic_hook!(71, GDT::KCODE, 0);
    create_generic_hook!(72, GDT::KCODE, 0);
    create_generic_hook!(73, GDT::KCODE, 0);
    create_generic_hook!(74, GDT::KCODE, 0);
    create_generic_hook!(75, GDT::KCODE, 0);
    create_generic_hook!(76, GDT::KCODE, 0);
    create_generic_hook!(77, GDT::KCODE, 0);
    create_generic_hook!(78, GDT::KCODE, 0);
    create_generic_hook!(79, GDT::KCODE, 0);
    create_generic_hook!(80, GDT::KCODE, 0);
    create_generic_hook!(81, GDT::KCODE, 0);
    create_generic_hook!(82, GDT::KCODE, 0);
    create_generic_hook!(83, GDT::KCODE, 0);
    create_generic_hook!(84, GDT::KCODE, 0);
    create_generic_hook!(85, GDT::KCODE, 0);
    create_generic_hook!(86, GDT::KCODE, 0);
    create_generic_hook!(87, GDT::KCODE, 0);
    create_generic_hook!(88, GDT::KCODE, 0);
    create_generic_hook!(89, GDT::KCODE, 0);
    create_generic_hook!(90, GDT::KCODE, 0);
    create_generic_hook!(91, GDT::KCODE, 0);
    create_generic_hook!(92, GDT::KCODE, 0);
    create_generic_hook!(93, GDT::KCODE, 0);
    create_generic_hook!(94, GDT::KCODE, 0);
    create_generic_hook!(95, GDT::KCODE, 0);
    create_generic_hook!(96, GDT::KCODE, 0);
    create_generic_hook!(97, GDT::KCODE, 0);
    create_generic_hook!(98, GDT::KCODE, 0);
    create_generic_hook!(99, GDT::KCODE, 0);
    create_generic_hook!(100, GDT::KCODE, 0);
    create_generic_hook!(101, GDT::KCODE, 0);
    create_generic_hook!(102, GDT::KCODE, 0);
    create_generic_hook!(103, GDT::KCODE, 0);
    create_generic_hook!(104, GDT::KCODE, 0);
    create_generic_hook!(105, GDT::KCODE, 0);
    create_generic_hook!(106, GDT::KCODE, 0);
    create_generic_hook!(107, GDT::KCODE, 0);
    create_generic_hook!(108, GDT::KCODE, 0);
    create_generic_hook!(109, GDT::KCODE, 0);
    create_generic_hook!(110, GDT::KCODE, 0);
    create_generic_hook!(111, GDT::KCODE, 0);
    create_generic_hook!(112, GDT::KCODE, 0);
    create_generic_hook!(113, GDT::KCODE, 0);
    create_generic_hook!(114, GDT::KCODE, 0);
    create_generic_hook!(115, GDT::KCODE, 0);
    create_generic_hook!(116, GDT::KCODE, 0);
    create_generic_hook!(117, GDT::KCODE, 0);
    create_generic_hook!(118, GDT::KCODE, 0);
    create_generic_hook!(119, GDT::KCODE, 0);
    create_generic_hook!(120, GDT::KCODE, 0);
    create_generic_hook!(121, GDT::KCODE, 0);
    create_generic_hook!(122, GDT::KCODE, 0);
    create_generic_hook!(123, GDT::KCODE, 0);
    create_generic_hook!(124, GDT::KCODE, 0);
    create_generic_hook!(125, GDT::KCODE, 0);
    create_generic_hook!(126, GDT::KCODE, 0);
    create_generic_hook!(127, GDT::KCODE, 0);
    create_generic_hook!(128, GDT::KCODE, 0);
    create_generic_hook!(129, GDT::KCODE, 0);
    create_generic_hook!(130, GDT::KCODE, 0);
    create_generic_hook!(131, GDT::KCODE, 0);
    create_generic_hook!(132, GDT::KCODE, 0);
    create_generic_hook!(133, GDT::KCODE, 0);
    create_generic_hook!(134, GDT::KCODE, 0);
    create_generic_hook!(135, GDT::KCODE, 0);
    create_generic_hook!(136, GDT::KCODE, 0);
    create_generic_hook!(137, GDT::KCODE, 0);
    create_generic_hook!(138, GDT::KCODE, 0);
    create_generic_hook!(139, GDT::KCODE, 0);
    create_generic_hook!(140, GDT::KCODE, 0);
    create_generic_hook!(141, GDT::KCODE, 0);
    create_generic_hook!(142, GDT::KCODE, 0);
    create_generic_hook!(143, GDT::KCODE, 0);
    create_generic_hook!(144, GDT::KCODE, 0);
    create_generic_hook!(145, GDT::KCODE, 0);
    create_generic_hook!(146, GDT::KCODE, 0);
    create_generic_hook!(147, GDT::KCODE, 0);
    create_generic_hook!(148, GDT::KCODE, 0);
    create_generic_hook!(149, GDT::KCODE, 0);
    create_generic_hook!(150, GDT::KCODE, 0);
    create_generic_hook!(151, GDT::KCODE, 0);
    create_generic_hook!(152, GDT::KCODE, 0);
    create_generic_hook!(153, GDT::KCODE, 0);
    create_generic_hook!(154, GDT::KCODE, 0);
    create_generic_hook!(155, GDT::KCODE, 0);
    create_generic_hook!(156, GDT::KCODE, 0);
    create_generic_hook!(157, GDT::KCODE, 0);
    create_generic_hook!(158, GDT::KCODE, 0);
    create_generic_hook!(159, GDT::KCODE, 0);
    create_generic_hook!(160, GDT::KCODE, 0);
    create_generic_hook!(161, GDT::KCODE, 0);
    create_generic_hook!(162, GDT::KCODE, 0);
    create_generic_hook!(163, GDT::KCODE, 0);
    create_generic_hook!(164, GDT::KCODE, 0);
    create_generic_hook!(165, GDT::KCODE, 0);
    create_generic_hook!(166, GDT::KCODE, 0);
    create_generic_hook!(167, GDT::KCODE, 0);
    create_generic_hook!(168, GDT::KCODE, 0);
    create_generic_hook!(169, GDT::KCODE, 0);
    create_generic_hook!(170, GDT::KCODE, 0);
    create_generic_hook!(171, GDT::KCODE, 0);
    create_generic_hook!(172, GDT::KCODE, 0);
    create_generic_hook!(173, GDT::KCODE, 0);
    create_generic_hook!(174, GDT::KCODE, 0);
    create_generic_hook!(175, GDT::KCODE, 0);
    create_generic_hook!(176, GDT::KCODE, 0);
    create_generic_hook!(177, GDT::KCODE, 0);
    create_generic_hook!(178, GDT::KCODE, 0);
    create_generic_hook!(179, GDT::KCODE, 0);
    create_generic_hook!(180, GDT::KCODE, 0);
    create_generic_hook!(181, GDT::KCODE, 0);
    create_generic_hook!(182, GDT::KCODE, 0);
    create_generic_hook!(183, GDT::KCODE, 0);
    create_generic_hook!(184, GDT::KCODE, 0);
    create_generic_hook!(185, GDT::KCODE, 0);
    create_generic_hook!(186, GDT::KCODE, 0);
    create_generic_hook!(187, GDT::KCODE, 0);
    create_generic_hook!(188, GDT::KCODE, 0);
    create_generic_hook!(189, GDT::KCODE, 0);
    create_generic_hook!(190, GDT::KCODE, 0);
    create_generic_hook!(191, GDT::KCODE, 0);
    create_generic_hook!(192, GDT::KCODE, 0);
    create_generic_hook!(193, GDT::KCODE, 0);
    create_generic_hook!(194, GDT::KCODE, 0);
    create_generic_hook!(195, GDT::KCODE, 0);
    create_generic_hook!(196, GDT::KCODE, 0);
    create_generic_hook!(197, GDT::KCODE, 0);
    create_generic_hook!(198, GDT::KCODE, 0);
    create_generic_hook!(199, GDT::KCODE, 0);
    create_generic_hook!(200, GDT::KCODE, 0);
    create_generic_hook!(201, GDT::KCODE, 0);
    create_generic_hook!(202, GDT::KCODE, 0);
    create_generic_hook!(203, GDT::KCODE, 0);
    create_generic_hook!(204, GDT::KCODE, 0);
    create_generic_hook!(205, GDT::KCODE, 0);
    create_generic_hook!(206, GDT::KCODE, 0);
    create_generic_hook!(207, GDT::KCODE, 0);
    create_generic_hook!(208, GDT::KCODE, 0);
    create_generic_hook!(209, GDT::KCODE, 0);
    create_generic_hook!(210, GDT::KCODE, 0);
    create_generic_hook!(211, GDT::KCODE, 0);
    create_generic_hook!(212, GDT::KCODE, 0);
    create_generic_hook!(213, GDT::KCODE, 0);
    create_generic_hook!(214, GDT::KCODE, 0);
    create_generic_hook!(215, GDT::KCODE, 0);
    create_generic_hook!(216, GDT::KCODE, 0);
    create_generic_hook!(217, GDT::KCODE, 0);
    create_generic_hook!(218, GDT::KCODE, 0);
    create_generic_hook!(219, GDT::KCODE, 0);
    create_generic_hook!(220, GDT::KCODE, 0);
    create_generic_hook!(221, GDT::KCODE, 0);
    create_generic_hook!(222, GDT::KCODE, 0);
    create_generic_hook!(223, GDT::KCODE, 0);
    create_generic_hook!(224, GDT::KCODE, 0);
    create_generic_hook!(225, GDT::KCODE, 0);
    create_generic_hook!(226, GDT::KCODE, 0);
    create_generic_hook!(227, GDT::KCODE, 0);
    create_generic_hook!(228, GDT::KCODE, 0);
    create_generic_hook!(229, GDT::KCODE, 0);
    create_generic_hook!(230, GDT::KCODE, 0);
    create_generic_hook!(231, GDT::KCODE, 0);
    create_generic_hook!(232, GDT::KCODE, 0);
    create_generic_hook!(233, GDT::KCODE, 0);
    create_generic_hook!(234, GDT::KCODE, 0);
    create_generic_hook!(235, GDT::KCODE, 0);
    create_generic_hook!(236, GDT::KCODE, 0);
    create_generic_hook!(237, GDT::KCODE, 0);
    create_generic_hook!(238, GDT::KCODE, 0);
    create_generic_hook!(239, GDT::KCODE, 0);
    create_generic_hook!(240, GDT::KCODE, 0);
    create_generic_hook!(241, GDT::KCODE, 0);
    create_generic_hook!(242, GDT::KCODE, 0);
    create_generic_hook!(243, GDT::KCODE, 0);
    create_generic_hook!(244, GDT::KCODE, 0);
    create_generic_hook!(245, GDT::KCODE, 0);
    create_generic_hook!(246, GDT::KCODE, 0);
    create_generic_hook!(247, GDT::KCODE, 0);
    create_generic_hook!(248, GDT::KCODE, 0);
    create_generic_hook!(249, GDT::KCODE, 0);
    create_generic_hook!(250, GDT::KCODE, 0);
    create_generic_hook!(251, GDT::KCODE, 0);
    create_generic_hook!(252, GDT::KCODE, 0);
    create_generic_hook!(253, GDT::KCODE, 0);
    create_generic_hook!(254, GDT::KCODE, 0);
    create_generic_hook!(255, GDT::KCODE, 0);

    idt_o.load();
}
