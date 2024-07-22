use aarch64_cpu::asm::barrier::SY;
use core::hint;
use core::num::NonZeroU32;
use core::ptr::NonNull;
use aarch64_cpu::asm::barrier;
use crate::arch::drivers::{SerialDriver, SerialSuccess};
use volatile::{VolatileFieldAccess, VolatileRef};
use crate::arch::drivers::SerialSuccess::{ERetry, Success};

const ZYNQ_UART_SR_TXACTIVE: u32 = 1<<11;
const ZYNQ_UART_SR_TXFULL: u32 = 1<<4;
const ZYNQ_UART_SR_TXEMPTY: u32 = 1<<3;
const ZYNQ_UART_SR_RXEMPTY: u32 = 1<<1;

const ZYNQ_UART_CR_TX_EN : u32 = 1<<4;
const ZYNQ_UART_CR_RX_EN : u32 = 1<<2;
const ZYNQ_UART_CR_TXRST : u32 = 1<<1;
const ZYNQ_UART_CR_RXRST : u32 = 1<<0;

const ZYNQ_UART_MR_STOPMODE_2_BIT :u32 = 0x00000080;
const ZYNQ_UART_MR_STOPMODE_1_5_BIT :u32 = 0x00000040;
const ZYNQ_UART_MR_STOPMODE_1_BIT : u32 = 0x00000000;

const ZYNQ_UART_MR_PARITY_NONE :u32 = 0x00000020;
const ZYNQ_UART_MR_PARITY_ODD :u32 = 0x00000008;
const ZYNQ_UART_MR_PARITY_EVEN :u32 = 0x00000000;

const ZYNQ_UART_MR_CHARLEN_6_BIT : u32 = 0x00000006;
const ZYNQ_UART_MR_CHARLEN_7_BIT : u32 = 0x00000004;
const ZYNQ_UART_MR_CHARLEN_8_BIT : u32 = 0x00000000;

#[repr(C)]
#[derive(VolatileFieldAccess)]
pub struct XlnxRegisters {
    control: u32,
    mode: u32,
    reserved1: [u32; 4],
    baud_rate_gen: u32,
    reserved2: [u32; 4],
    channel_sts: u32,
    tx_rx_fifo: u32,
    baud_rate_divider: u32,
}

pub struct XlnxSerial {
    regs: VolatileRef<'static, XlnxRegisters>,
}

impl XlnxSerial {
    pub fn from_addr(base_addr: NonZeroU32) -> XlnxSerial {
        Self {regs: unsafe { VolatileRef::new( NonNull::new_unchecked(base_addr.get() as *mut XlnxRegisters) )} }
    }
}

impl SerialDriver for XlnxSerial {
    fn init(&mut self)  {
        self.regs.as_mut_ptr().control().write(ZYNQ_UART_CR_TX_EN | ZYNQ_UART_CR_RX_EN | ZYNQ_UART_CR_TXRST | ZYNQ_UART_CR_RXRST);
        self.regs.as_mut_ptr().mode().write(ZYNQ_UART_MR_PARITY_NONE);
    }

    fn set_baud(&self, _baud: u32) {return;}

    fn putc(&mut self, c: u8) -> SerialSuccess<u8> {
        barrier::dmb(SY);
        barrier::dsb(SY);
        if self.regs.as_mut_ptr().channel_sts().read() & ZYNQ_UART_SR_TXFULL != 0 { return ERetry; }

        self.regs.as_mut_ptr().tx_rx_fifo().write(c as u32);
        Success(c)
    }

    fn getc(&self) -> SerialSuccess<u8> {
        Success('A' as u8)
    }

    fn putstr(&mut self, s: &[u8]) {
        'foo: for c in s.iter().copied() {
            if c == b'\n' {
                for _ in 0..1000 {
                    match self.putc(b'\r') {
                        ERetry => continue,
                        Success(c) => break
                    }
                }
            }
            for _ in 0..1000 {
                hint::spin_loop();
                match self.putc(c) {
                    ERetry => continue,
                    Success(_) => continue 'foo,
                }
            }
            self.init();
            while self.regs.as_mut_ptr().channel_sts().read() & ZYNQ_UART_SR_TXEMPTY != 0 {hint::spin_loop();}
        }
    }

    fn get_addr(&self) -> u32 {
        self.regs.as_ptr().as_raw_ptr().as_ptr() as u32 
    }

    fn wait_empty(&mut self) {
        while self.regs.as_mut_ptr().channel_sts().read() & ZYNQ_UART_SR_TXACTIVE != 0 {
            hint::spin_loop();
        }
        while self.regs.as_mut_ptr().channel_sts().read() & ZYNQ_UART_SR_TXEMPTY != 0 {
            hint::spin_loop();
        }
    }
}



