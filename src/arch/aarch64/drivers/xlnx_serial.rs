use crate::arch::aarch64::console::SerialDriver;
use volatile::{VolatileFieldAccess, VolatileRef};

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
struct XlnxSerial {
    control: u32,
    mode: u32,
    reserved: u32,
    baud_rate_gen: u32,
    reserved2: u32,
    channel_sts: u32,
    tx_rx_fifo: u32,
    baud_rate_divider: u32,
}

impl XlnxSerial {
    pub fn from_addr(base_addr: u32) -> &'static mut XlnxSerial {
        unsafe { &mut *(base_addr as *mut XlnxSerial) }
    }
}

impl SerialDriver for XlnxSerial {
    fn init(&mut self) {
        let mut volatile_ref = VolatileRef::from_mut_ref(self);
        let volatile_ptr = volatile_ref.as_mut_ptr();
        volatile_ptr.control().write(ZYNQ_UART_CR_TX_EN | ZYNQ_UART_CR_RX_EN | ZYNQ_UART_CR_TXRST | ZYNQ_UART_CR_RXRST);
        volatile_ptr.mode().write(ZYNQ_UART_MR_PARITY_NONE);
    }

    fn set_baud(&self, _baud: u32) {return;}

    fn putc(&mut self, c: u8) {
        let mut volatile_ref = VolatileRef::from_mut_ref(self);
        let volatile_ptr = volatile_ref.as_mut_ptr();

        if (volatile_ptr.channel_sts().read() & ZYNQ_UART_SR_TXFULL != 0) { return; }

        volatile_ptr.tx_rx_fifo().write(c as u32);
    }

    fn getc(&self) -> u8 {
        'A' as u8
    }

    fn get_addr(&self) -> u32 {
        unsafe { core::ptr::addr_of!(*self) as u32 }
    }
}



