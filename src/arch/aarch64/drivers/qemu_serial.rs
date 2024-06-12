use core::ptr::NonNull;
use crate::arch::aarch64::console::SerialDriver;
use volatile_register::{RW, RO};

#[repr(C)]
pub struct QemuSerial {
    pub out: RW<u8>,
}

impl QemuSerial {
    pub fn from_addr(base_addr: u32) -> &'static mut QemuSerial {
        unsafe { &mut *(base_addr as *mut QemuSerial) }
    }
}

impl SerialDriver for QemuSerial {
    fn init(&self) {return;}
    fn set_baud(&self, baud_rate: u32) {return;}
    fn putc(&self, c: u8) {
        unsafe {
            self.out.write(c);
        }
    }
    fn getc(&self) -> u8 {
        'A' as u8
    }
    fn get_addr(&self) -> u32 {
        unsafe { core::ptr::addr_of!(self) as u32 }
    }
}
