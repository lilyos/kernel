/// Possible types of interrupts
#[repr(C)]
#[derive(Debug)]
pub enum InterruptType {
    /// Self explanatory
    DivideByZero(DivideByZeroContext),
    /// An interrupt originates from a breakpoint instruction
    DebugBreakpoint(DebugBreakpointContext),
    /// We don't know what this is
    Generic(GenericContext),
    /// An invalid instruction was encountered
    InvalidInstruction(InvalidInstructionContext),
    /// An illegal address was accessed, could be from a lack of privilege or it just being unmapped
    IllegalAccess(IllegalAccessContext),
    /// Some processor-specific structure was invalid,
    /// such as the TSS, GDT, or a Segment on x86_64
    InvalidProcessorStructure(NoHopeContext),
    /// Some type of hardware check failed
    CheckFailed(CheckFailedContext),
    /// A SIMD error occurred
    SIMDError(SIMDErrorContext),
    /// An error related to the FPU
    FloatingPoint(FloatingPointContext),
    /// An error related to Virtualization occurred
    VirtualizationError(VirtualizationErrorContext),
    /// The hypervisor is attempting to interfere with the guest,
    /// refers to `VMMCommunicationException`, `HypervisorInjectionException`,
    /// and `SecurityException`
    HypervisorInterference(HypervisorInterferenceContext),
    /// A processor security feature was subverted, such as the violation of a shadow stack or branch prediction
    ControlProtectionViolation(ControlProtectionContext),
    /// This cannot be interrupted, such as double fault or NMI on Arm
    NonMaskableInterrupt(NonMaskableInterruptContext),
}

#[repr(C)]
#[derive(Debug)]
/// Context for a DivideByZero exception
pub struct DivideByZeroContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for a DebugBreakpoint exception
pub struct DebugBreakpointContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for a Generic exceptions
pub struct GenericContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// The interrupt's number
    pub interrupt_number: u64,
    /// Optional Error Code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for an InvalidInstruction exception
pub struct InvalidInstructionContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for an IllegalAccess exception
pub struct IllegalAccessContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// If this was false, it was an attempt to read a privileged area
    pub page_unmapped: bool,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Use this when you're fucked, there is no recovering
pub struct NoHopeContext {
    /// The message you can see before the kernel dies
    pub message: &'static str,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for a CheckFailed exception
pub struct CheckFailedContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// The message for what check failed
    pub message: &'static str,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for a SIMDError exception
pub struct SIMDErrorContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for a FloatingPoint exception
pub struct FloatingPointContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for a Virtualization exception
pub struct VirtualizationErrorContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for a HypervisorInterference exception
pub struct HypervisorInterferenceContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for a ControlProtectionViolation exception
pub struct ControlProtectionContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// Optional error code
    pub error_code: Option<u64>,
}

#[repr(C)]
#[derive(Debug)]
/// Context for a NonMaskableInterrupt exception
pub struct NonMaskableInterruptContext {
    /// The ID of the offending process
    pub pid: u64,
    /// The instruction pointer
    pub iptr: *mut u8,
    /// Optional error code
    pub error_code: Option<u64>,
}
