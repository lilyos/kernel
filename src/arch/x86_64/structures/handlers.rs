use crate::{
    interrupts::{
        CheckFailedContext, ControlProtectionContext, DebugBreakpointContext, DivideByZeroContext,
        FloatingPointContext, GenericContext, HypervisorInterferenceContext, IllegalAccessContext,
        InterruptType, InvalidInstructionContext, NonMaskableInterruptContext, SIMDErrorContext,
        VirtualizationErrorContext,
    },
    macros::bitflags::bitflags,
};

use super::{ExceptionStackFrame, INTERRUPT_HANDLER};

macro_rules! invoke_handler {
    ($ctx:expr) => {
        unsafe { INTERRUPT_HANDLER.expect("INTERRUPT HANDLER NOT INSTALLED")($ctx) }
    };
}

/// DivideByZero hook
pub extern "x86-interrupt" fn divide_by_zero(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::DivideByZero(DivideByZeroContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        error_code: None,
    }))
}

/// DebugBreakpoint hook
pub extern "x86-interrupt" fn debug(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::DebugBreakpoint(DebugBreakpointContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        error_code: None,
    }))
}

/// DebugBreakpoint hook
pub extern "x86-interrupt" fn breakpoint(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::DebugBreakpoint(DebugBreakpointContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        error_code: None,
    }))
}

/// Generic hook
pub extern "x86-interrupt" fn general_protection(frame: &mut ExceptionStackFrame, error_code: u64) {
    invoke_handler!(InterruptType::Generic(GenericContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        interrupt_number: 13,
        error_code: Some(error_code),
    }))
}

/// Generic hook
pub extern "x86-interrupt" fn overflow(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::Generic(GenericContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        interrupt_number: 4,
        error_code: None,
    }))
}

/// Generic hook
pub extern "x86-interrupt" fn bound_range_exceeded(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::Generic(GenericContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        interrupt_number: 5,
        error_code: None,
    }))
}

/// InvalidInstruction hook
pub extern "x86-interrupt" fn invalid_opcode(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::InvalidInstruction(
        InvalidInstructionContext {
            pid: 0,
            iptr: frame.instruction_pointer,
            error_code: None,
        }
    ))
}

bitflags! {
    struct PageFaultErrorCode: u64 {
        /// If page was present
        const PRESENT = 1 << 0;
        /// If set, it was caused by writing, else reading
        const WRITE = 1 << 1;
        /// The fault came from Privilege level 3
        const USER = 1 << 2;
        /// When set it indicates one or more page directory entries contain reserved bits which are set to one
        const RESERVED_WRITE = 1 << 3;
        /// When set it indicates the fault was caused by an instruction fetch
        const INSTRUCTION_FETCH = 1 << 4;
        /// When set it indicates a protection-key violation
        const PROTECTION_KEY = 1 << 5;
        /// When set it indicates the page fault was caused by a shadow stack access
        const SHADOW_STACK = 1 << 6;
        /// When set the fault was due to an SGX violation
        const SGX = 1 << 15;
    }
}

/// InvalidAccess hook
pub extern "x86-interrupt" fn page_fault(frame: &mut ExceptionStackFrame, error_code: u64) {
    invoke_handler!(InterruptType::IllegalAccess(IllegalAccessContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        page_unmapped: PageFaultErrorCode::from_bits_truncate(error_code)
            .contains(PageFaultErrorCode::PRESENT),
        error_code: Some(error_code),
    }))
}

/// CheckFailed hook
pub extern "x86-interrupt" fn alignment(frame: &mut ExceptionStackFrame, error_code: u64) {
    invoke_handler!(InterruptType::CheckFailed(CheckFailedContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        message: "FAILED ALIGNMENT CHECK",
        error_code: Some(error_code),
    }))
}

/// CheckFailed hook
pub extern "x86-interrupt" fn machine(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::CheckFailed(CheckFailedContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        message: "FAILED MACHINE CHECK",
        error_code: None,
    }))
}

/// CheckFailed hook
pub extern "x86-interrupt" fn device_not_available(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::CheckFailed(CheckFailedContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        message: "DEVICE NOT AVAILABLE",
        error_code: None,
    }))
}

/// CheckFailed hook
pub extern "x86-interrupt" fn invalid_tss(frame: &mut ExceptionStackFrame, error_code: u64) {
    invoke_handler!(InterruptType::CheckFailed(CheckFailedContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        message: "FAILED TO VERIFY TSS",
        error_code: Some(error_code),
    }))
}

/// CheckFailed hook
pub extern "x86-interrupt" fn segment_not_present(
    frame: &mut ExceptionStackFrame,
    error_code: u64,
) {
    invoke_handler!(InterruptType::CheckFailed(CheckFailedContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        message: "FAILED TO SET SEGMENT",
        error_code: Some(error_code),
    }))
}

/// CheckFailed hook
pub extern "x86-interrupt" fn stack_segment_fault(
    frame: &mut ExceptionStackFrame,
    error_code: u64,
) {
    invoke_handler!(InterruptType::CheckFailed(CheckFailedContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        message: "FAILED TO SET STACK SEGMENT",
        error_code: Some(error_code),
    }))
}

/// SimdError hook
pub extern "x86-interrupt" fn simd_floating_point(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::SIMDError(SIMDErrorContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        error_code: None,
    }))
}

/// FloatingPoint hook
pub extern "x86-interrupt" fn floating_point(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::FloatingPoint(FloatingPointContext {
        pid: 0,
        iptr: frame.instruction_pointer,
        error_code: None,
    }))
}

/// VirtualizationError hook
pub extern "x86-interrupt" fn virtualization(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::VirtualizationError(
        VirtualizationErrorContext {
            pid: 0,
            iptr: frame.instruction_pointer,
            error_code: None,
        }
    ))
}

/// VirtalizationError hook
pub extern "x86-interrupt" fn vmm_communication(frame: &mut ExceptionStackFrame, error_code: u64) {
    invoke_handler!(InterruptType::VirtualizationError(
        VirtualizationErrorContext {
            pid: 0,
            iptr: frame.instruction_pointer,
            error_code: Some(error_code),
        }
    ))
}

/// HypervisorInterference hook
pub extern "x86-interrupt" fn hypervisor_injection(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::HypervisorInterference(
        HypervisorInterferenceContext {
            pid: 0,
            iptr: frame.instruction_pointer,
            error_code: None,
        }
    ))
}

/// ControlProtectionViolation hook
pub extern "x86-interrupt" fn control_protection(frame: &mut ExceptionStackFrame, error_code: u64) {
    invoke_handler!(InterruptType::ControlProtectionViolation(
        ControlProtectionContext {
            pid: 0,
            iptr: frame.instruction_pointer,
            error_code: Some(error_code),
        }
    ))
}

/// ControlProtectionViolation hook
pub extern "x86-interrupt" fn security_violation(frame: &mut ExceptionStackFrame, error_code: u64) {
    invoke_handler!(InterruptType::ControlProtectionViolation(
        ControlProtectionContext {
            pid: 0,
            iptr: frame.instruction_pointer,
            error_code: Some(error_code),
        }
    ))
}

/// NonMaskableInterrupt hook
pub extern "x86-interrupt" fn nmi(frame: &mut ExceptionStackFrame) {
    invoke_handler!(InterruptType::NonMaskableInterrupt(
        NonMaskableInterruptContext {
            pid: 0,
            iptr: frame.instruction_pointer,
            error_code: None,
        }
    ))
}

/// NonMaskableInterrupt hook
pub extern "x86-interrupt" fn double_fault(frame: &mut ExceptionStackFrame, error_code: u64) {
    invoke_handler!(InterruptType::NonMaskableInterrupt(
        NonMaskableInterruptContext {
            pid: 0,
            iptr: frame.instruction_pointer,
            error_code: Some(error_code),
        }
    ))
}
