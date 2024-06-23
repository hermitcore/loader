use core::num::NonZeroU32;
use hermit_dtb::Dtb;

use crate::arch::drivers::xlnx_serial::XlnxSerial;
use crate::arch::drivers::SerialDriver;

pub struct Console {
	stdout: XlnxSerial,
}

pub fn stdout() -> XlnxSerial {
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
		} else if let Some(pos) = stdout.find(':') {
			let alias = stdout.split_at(pos).0;
			let txt = dtb.get_property("/aliases", alias);
			if let Some(port) = txt {
				let port = core::str::from_utf8(port).unwrap();
				if let Some(pos) = port.find('@') {
					let len = stdout.len();
					u32::from_str_radix(&stdout[pos + 1..len], 16).unwrap_or(SERIAL_PORT_ADDRESS)
				} else { 
					SERIAL_PORT_ADDRESS
				}
			} else {
				SERIAL_PORT_ADDRESS
			}
		} else {
			SERIAL_PORT_ADDRESS
		}
	} else {
		SERIAL_PORT_ADDRESS
	};
	let mut  serial = XlnxSerial::from_addr(NonZeroU32::new(0xff000000).unwrap());
	serial.init();
	serial
}

impl Console {
	pub fn write_bytes(&mut self, bytes: &[u8]) {
		self.stdout.putstr(bytes);
	}

	pub(super) fn get_stdout(&self) -> u32 {
		self.stdout.get_addr()
	}

	pub(super) fn set_stdout(&mut self, stdout: u32) {
		self.stdout = XlnxSerial::from_addr(NonZeroU32::new(stdout).unwrap());
	}
}

impl Default for Console {
	fn default() -> Self {
		let stdout = stdout();
		Self { stdout }
	}
}

unsafe impl Send for Console {}
