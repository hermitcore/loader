use core::num::NonZeroU32;
use core::ptr::NonNull;
use crate::arch::drivers::{SerialDriver, SerialSuccess};
use volatile::{VolatileFieldAccess, VolatileRef};
use crate::arch::drivers::SerialSuccess::Success;

#[repr(C)]
#[derive(VolatileFieldAccess)]
struct QemuPort {
    out: u8,
}

pub struct QemuSerial {
    regs: VolatileRef<'static, QemuPort>
}


impl QemuSerial {
    pub fn from_addr(base_addr: NonZeroU32) -> QemuSerial {
        Self {regs: unsafe { VolatileRef::new( NonNull::new_unchecked(base_addr.get() as *mut QemuPort) )} }
    }
}

impl SerialDriver for QemuSerial {
    fn init(&mut self) {return;}
    fn set_baud(&self, baud_rate: u32) {return;}
    fn putc(&mut self, c: u8) -> SerialSuccess<u8>{
        self.regs.as_mut_ptr().out().write(c);
        Success(c)
    }
    fn getc(&self) -> SerialSuccess<u8> {
        Success('A' as u8)
    }

    fn putstr(&mut self, s: &[u8]) {
        for c in s.iter().copied() {
            let _ = self.putc(c);
        }
    }
    fn get_addr(&self) -> u32 {
        self.regs.as_ptr().as_raw_ptr().as_ptr() as u32
    }
}
