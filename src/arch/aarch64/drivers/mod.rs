pub mod qemu_serial;
pub mod xlnx_serial;

pub enum SerialSuccess<T> {
    Success(T),
    ERetry
}
pub trait SerialDriver {
    fn init(&mut self);
    fn set_baud(&self, baud_rate: u32);
    fn putc(&mut self, c: u8) -> SerialSuccess<u8>;
    fn getc(&self) -> SerialSuccess<u8>;
    fn putstr(&mut self, s: &[u8]);
    fn get_addr(&self) -> u32;
}