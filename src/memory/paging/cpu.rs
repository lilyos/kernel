use kernel_macros::bit_field_accessors;

#[repr(transparent)]
pub struct CR0(pub u64);

impl CR0 {
    bit_field_accessors! {
        protected_mode 0;
        monitor_coprocessor 1;
        emulation 2;
        task_switched 3;
        extension_type 4;
        numeric_error 5;
        write_protect 16;
        alignment_mask 18;
        not_write_through 29;
        cache_disable 30;
        paging 31;
    }
}
