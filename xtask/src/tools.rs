use duct::cmd;

use bitflags::bitflags;

bitflags! {
    pub struct Tools: u64 {
        const GIT = 1 << 0;
        const QEMU_X86_64: 1 << 1;
        
    }
}

pub fn get_tools() -> Tools {
    let mut Tools = Tools::empty();

    if let Ok(_) = cmd!("git", "--version").stdout_null().run() {
        Tools |= 
    }
}