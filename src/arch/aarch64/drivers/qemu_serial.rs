use crate::arch::aarch64::console::SerialDriver;
use volatile::{VolatileFieldAccess, VolatileRef};

#[repr(C)]
#[derive(VolatileFieldAccess)]
pub struct QemuSerial {
    pub out: u8,
}

impl QemuSerial {
    pub fn from_addr(base_addr: u32) -> &'static mut QemuSerial {
        unsafe { &mut *(base_addr as *mut QemuSerial) }
    }
}

impl SerialDriver for QemuSerial {
    fn init(&mut self) {return;}
    fn set_baud(&self, baud_rate: u32) {return;}
    fn putc(&mut self, c: u8) {
        let mut volatile_ref = VolatileRef::from_mut_ref(self);
        let volatile_ptr = volatile_ref.as_mut_ptr();
        unsafe {
            volatile_ptr.out().write(c);
        }
    }
    fn getc(&self) -> u8 {
        'A' as u8
    }
    fn get_addr(&self) -> u32 {
        unsafe { core::ptr::addr_of!(*self) as u32 }
    }
}
