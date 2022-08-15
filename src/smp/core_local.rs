use core::alloc::Layout;

use crate::{
    errors::MemoryManagerError,
    get_memory_manager,
    memory::allocators::NeverAllocator,
    sync::RwLock,
    traits::{Init, MemoryFlags, MemoryManager},
};

#[repr(C, align(0x1000))]
pub struct CoreLocalData {
    pub magic: u32,
    pub id: u32,
    pub heap: *mut (),
    pub scheduler: *mut (),
    platform_data: Box<[u8]>,
}

pub struct CoreManager {
    core_refs: RwLock<Vec<(u32, *mut CoreLocalData), NeverAllocator>>,
}

impl CoreManager {
    pub const fn new() -> Self {
        Self {
            core_refs: RwLock::new(Vec::new_in(NeverAllocator)),
        }
    }

    pub fn get_core_local_data(&self, id: u32) -> Option<*mut CoreLocalData> {
        Some(self.core_refs.read().iter().find(|(i, _)| *i == id)?.1)
    }

    pub fn initialize_core(&self) {}
}

impl Init for CoreManager {
    type Error = MemoryManagerError;

    type Input = usize;

    fn init(&self, init_val: Self::Input) -> Result<(), Self::Error> {
        let mut region = unsafe {
            get_memory_manager().allocate_and_map(
                get_memory_manager().get_current_table()?,
                (*crate::SAFE_UPPER_HALF_RANGE).clone(),
                MemoryFlags::KERNEL_ONLY | MemoryFlags::READABLE | MemoryFlags::WRITABLE,
                Layout::from_size_align_unchecked(
                    core::mem::size_of::<(u32, CoreLocalData)>() * init_val,
                    core::mem::align_of::<(u32, CoreLocalData)>(),
                ),
            )
        }?;

        {
            let mut data = self.core_refs.write();
            *data = unsafe {
                Vec::from_raw_parts_in(
                    region.get_inner_ptr_mut() as *mut (u32, *mut CoreLocalData),
                    0,
                    init_val,
                    NeverAllocator,
                )
            };
        }

        Ok(())
    }
}
