use core::ptr::NonNull;

use hermit_dtb::Dtb;

use crate::arch::drivers::qemu_serial::QemuSerial;

pub struct Console {
	stdout: &'static dyn SerialDriver,
}

pub trait SerialDriver {
    fn init(&self);
    fn set_baud(&self, baud_rate: u32);
    fn putc(&self, c: u8);
    fn getc(&self) -> u8;
    fn get_addr(&self) -> u32;
}

fn stdout() -> impl SerialDriver {
	/// Physical address of UART0 at Qemu's virt emulation
	const SERIAL_PORT_ADDRESS: u32 = 0x09000000;

	let dtb = unsafe {
		Dtb::from_raw(sptr::from_exposed_addr(super::DEVICE_TREE as usize))
			.expect(".dtb file has invalid header")
	};

	let property = dtb.get_property("/chosen", "stdout-path");
	let uart_address = if let Some(stdout) = property {
		let stdout = core::str::from_utf8(stdout)
			.unwrap()
			.trim_matches(char::from(0));
		if let Some(pos) = stdout.find('@') {
			let len = stdout.len();
			u32::from_str_radix(&stdout[pos + 1..len], 16).unwrap_or(SERIAL_PORT_ADDRESS)
		} else {
			SERIAL_PORT_ADDRESS
		}
	} else {
		SERIAL_PORT_ADDRESS
	};
	QemuSerial::from_addr(uart_address)
}

impl Console {
	pub fn write_bytes(&mut self, bytes: &[u8]) {
		for byte in bytes.iter().copied() {
			unsafe {
				self.stdout.putc(byte);
			}
		}
	}

	pub(super) fn get_stdout(&self) -> impl SerialDriver {
		self.stdout
	}

	pub(super) fn set_stdout(&mut self, stdout: &'static dyn SerialDriver) {
		self.stdout = stdout;
	}
}

impl Default for Console {
	fn default() -> Self {
		let stdout = stdout();
		Self { stdout }
	}
}

unsafe impl Send for Console {}
