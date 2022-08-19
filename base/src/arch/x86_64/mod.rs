use log::info;

use crate::traits::{Init, Platform};

use self::{
    interrupt_manager::InterruptManager,
    memory::{addresses::RawAddress, memory_manager::MemoryManager},
    peripherals::{TimerManager, Uart},
    power_manager::PowerManager,
};

/// Architecture-specific structures, such as the IDT or GDT
pub mod structures;

/// Architecture-specific code relating to memory management and virtual memory
pub mod memory;

/// Peripherals
pub mod peripherals;

pub mod interrupt_manager;

mod power_manager;

pub struct X86_64 {
    memory_manager: MemoryManager,
    interrupt_manager: InterruptManager,
    power_manager: PowerManager,
}

impl X86_64 {
    const fn new() -> Self {
        Self {
            memory_manager: MemoryManager::new(),
            interrupt_manager: InterruptManager::new(),
            power_manager: PowerManager::new(),
        }
    }
}

unsafe impl Platform for X86_64 {
    type MemoryManager = MemoryManager;

    type InterruptManager = InterruptManager;

    type PowerManager = PowerManager;

    type TimerManager = TimerManager;

    type RawAddress = RawAddress;

    type TextOutput = Uart;

    fn get_memory_manager(&'static self) -> &'static Self::MemoryManager {
        &self.memory_manager
    }

    fn get_interrupt_manager(&'static self) -> &'static Self::InterruptManager {
        &self.interrupt_manager
    }

    fn get_power_manager(&'static self) -> &'static Self::PowerManager {
        &self.power_manager
    }

    fn get_text_output(&'static self) -> &'static mut Self::TextOutput {
        // UART.get()
        // TODO: Add a ring buffer-ish solution, to prevent locking between kernel threads
        todo!()
    }

    fn initialize_current_core(&'static self) {
        todo!()
    }

    fn get_core_local(&'static self) -> &'static crate::smp::CoreLocalData {
        todo!()
    }
}

#[derive(Debug)]
pub enum X86_64InitError {
    MemoryManager(<<X86_64 as Platform>::MemoryManager as Init>::Error),
    InterruptManager(<<X86_64 as Platform>::InterruptManager as Init>::Error),
    PowerManager(<<X86_64 as Platform>::PowerManager as Init>::Error),
}

impl Init for X86_64 {
    type Error = X86_64InitError;

    type Input = ();

    fn init(&self, _: Self::Input) -> Result<(), Self::Error> {
        info!("Initializing Memory Manager");
        if let Err(e) = self.memory_manager.init(()) {
            return Err(X86_64InitError::MemoryManager(e));
        }

        info!("Initializing Interrupt Manager");
        if let Err(e) = self.interrupt_manager.init(()) {
            return Err(X86_64InitError::InterruptManager(e));
        }

        info!("Initializing Power Manager");
        if let Err(e) = self.power_manager.init(()) {
            return Err(X86_64InitError::PowerManager(e));
        }

        Ok(())
    }
}

/// [Platform] implementation for the x86_64 architecture
pub static IMPLEMENTATION: X86_64 = X86_64::new();
