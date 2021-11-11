#![allow(dead_code)]

use core::ptr::{read_volatile, write_volatile};

use crate::traits::Init;

const MAILBOX_FULL: u32 = 0x80000000;

const MAILBOX_RESPONSE: u32 = 0x80000000;

const MAILBOX_EMPTY: u32 = 0x40000000;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Commands {
    Request = 0,
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Channel {
    Power = 0,
    Framebuffer = 1,
    VUart = 2,
    VChiq = 3,
    Leds = 4,
    Bins = 5,
    Touch = 6,
    Count = 7,
    Property = 8,
}

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Tags {
    Last = 0,

    FirmwareRevision = 0x00000001,
    Model = 0x00010001,
    BoardRevision = 0x00010002,
    MACAddress = 0x10003,
    SerialNumber = 0x10004,

    ArmMemory = 0x00010005,
    VideoCoreMemory = 0x00010006,

    Clocks = 0x00010007,
    CommandLine = 0x00050001,
    DMAChannels = 0x00060001,

    GetPowerState = 0x00020001,
    PowerTiming = 0x00020002,

    SetPowerState = 0x00028001,
    GetClockState = 0x00030001,
    SetClockState = 0x00038001,

    GetClockRate = 0x00030002,
    GetTrueClockRate = 0x00030047,
    SetClockRate = 0x00038002,
    GetMaxClockRate = 0x00030004,
    GetMinClockRate = 0x00030007,
    GetTurbo = 0x00030009,
    SetTurbo = 0x00038009,

    GetVoltage = 0x00030003,
    SetVoltage = 0x00038003,
    GetMaxVoltage = 0x00030005,
    GetMinVoltage = 0x00030008,

    GetTemperature = 0x00030006,
    GetMaxTemperature = 0x0003000a,

    AllocateMemory = 0x0003000c,
    LockMemory = 0x0003000d,
    UnlockMemory = 0x0003000e,
    ReleaseMemory = 0x0003000f,

    ExecuteCode = 0x00030010,
    GetDispmanxResource = 0x00030014,
    GetEDID = 0x00030020,

    AllocateBuffer = 0x00040001,
    ReleaseBuffer = 0x00048001,
    BlankScreen = 0x00040002,

    GetDisplayWidthHeight = 0x00040003,
    TestDisplayWidthHeight = 0x00044003,
    SetDisplayWidthHeight = 0x00048003,

    GetBufferWidthHeight = 0x00040004,
    TestBufferWidthHeight = 0x00044004,
    SetBufferWidthHeight = 0x00048004,

    GetBitDepth = 0x00040005,
    TestBitDepth = 0x00044005,
    SetBitDepth = 0x00048005,

    GetPixelOrder = 0x00040006,
    TestPixelOrder = 0x00044006,
    SetPixelOrder = 0x00048006,

    GetAlphaMode = 0x00040007,
    TestAlphaMode = 0x00044007,
    SetAlphaMode = 0x00048007,

    GetPitch = 0x00040008,

    GetVirtualOffset = 0x00040009,
    TestVirtualOffset = 0x00044009,
    SetVirtualOffset = 0x00048009,

    GetOverscan = 0x0004000a,
    TestOverscan = 0x0004400a,
    SetOverscan = 0x0004800a,

    GetPalette = 0x0004000b,
    TestPalette = 0x0004400b,
    SetPalette = 0x0004800b,

    SetCursorInfo = 0x00008010,
    SetCursorState = 0x00008011,
}

#[repr(align(16), C)]
pub struct MailboxMessage(pub [u32; 36], pub Channel);

impl MailboxMessage {
    pub fn new() -> Self {
        MailboxMessage([0; 36], Channel::Property)
    }

    pub fn new_exact(ch: Channel, req: Tags, len: u32) -> Self {
        let mut tmp = [0; 36];
        tmp[0] = Commands::Request as u32;
        tmp[1] = req as u32;
        tmp[2] = 0;
        tmp[len as usize - 1usize] = Tags::Last as u32;
        MailboxMessage(tmp, ch)
    }
}

#[derive(Debug)]
pub struct Mailbox {
    base: *const u32,
    read: *const u32,
    poll: *const u32,
    sender: *const u32,
    status: *const u32,
    config: *const u32,
    write: *mut u32,
}

impl Mailbox {
    pub const fn new() -> Self {
        Mailbox {
            base: 0 as *const u32,
            read: 0 as *const u32,
            poll: 0 as *const u32,
            sender: 0 as *const u32,
            status: 0 as *const u32,
            config: 0 as *const u32,
            write: 0 as *mut u32,
        }
    }

    pub fn send(&mut self, msg: &mut MailboxMessage) -> bool {
        let msg_ptr = msg.0.as_ptr();
        let fnl_ptr = (msg_ptr as u32) | (msg.1 as u32 & 0xF);
        loop {
            if self.writeable() {
                break;
            }
        }
        unsafe {
            write_volatile(self.write, fnl_ptr);
        }
        loop {
            if self.has_response() && fnl_ptr == unsafe { read_volatile(self.read) } {
                break msg.0[1] as u32 == MAILBOX_RESPONSE;
            }
        }
    }

    fn writeable(&self) -> bool {
        unsafe { (read_volatile(self.status) & MAILBOX_FULL) == 0 }
    }

    fn has_response(&self) -> bool {
        unsafe { (read_volatile(self.status) & MAILBOX_EMPTY) == 0 }
    }
}

impl Init for Mailbox {
    fn pre_init(&mut self) {
        let base = crate::info::peripheral_base() + 0x0000B880;
        self.base = base as *const u32;
        self.read = base as *const u32;
        self.poll = (base + 0x10) as *const u32;
        self.sender = (base + 0x14) as *const u32;
        self.status = (base + 0x18) as *const u32;
        self.config = (base + 0x1C) as *const u32;
        self.write = (base + 0x20) as *mut u32;
    }

    fn init(&mut self) {}

    fn post_init(&mut self) {}
}
