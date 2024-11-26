use alloc::format;
use alloc::vec::Vec;

use log::debug;
use log::info;
use multiboot::information::{Multiboot, MemoryType};
use pci_types::{EndpointHeader, PciAddress, PciHeader, MAX_BARS, Bar};
use vm_fdt::{FdtWriter, FdtWriterResult};

use super::pci::{PciConfigRegion, PCI_MAX_BUS_NUMBER, PCI_MAX_DEVICE_NUMBER};
use super::multiboot::{mb_info, Mem};

pub struct DeviceTree;

impl DeviceTree {
	pub fn create() -> FdtWriterResult<&'static [u8]> {
		let mut mem = Mem;
		let multiboot = unsafe { Multiboot::from_ptr(mb_info as u64, &mut mem).unwrap() };

		let all_regions = multiboot
			.memory_regions()
			.expect("Could not find a memory map in the Multiboot information");
		let ram_regions = all_regions.filter(|m| m.memory_type() == MemoryType::Available);

		let mut fdt = FdtWriter::new()?;

		let root_node = fdt.begin_node("")?;
		fdt.property_string("compatible", "linux,dummy-virt")?;
		fdt.property_u32("#address-cells", 0x2)?;
		fdt.property_u32("#size-cells", 0x2)?;
        fdt.property_u32("#interrupt-cells", 0x1)?;

		if let Some(cmdline) = multiboot.command_line() {
			let chosen_node = fdt.begin_node("chosen")?;
			fdt.property_string("bootargs", cmdline)?;
			fdt.end_node(chosen_node)?;
		}

		for m in ram_regions {
			let start_address = m.base_address();
			let length = m.length();

			let memory_node = fdt.begin_node(format!("memory@{:x}", start_address).as_str())?;
			fdt.property_string("device_type", "memory")?;
			fdt.property_array_u64("reg", &[start_address, length])?;
			fdt.end_node(memory_node)?;
		}

		debug!("Scanning PCI Busses 0 to {}", PCI_MAX_BUS_NUMBER - 1);

		let pci_node = fdt.begin_node("pci")?;
		fdt.property_string("device_type", "pci")?;

		// TODO: Address cells and size cells should be 3 and 2 respectively.
		fdt.property_u32("#address-cells", 0x1)?;
		fdt.property_u32("#size-cells", 0x1)?;

		// Hermit only uses PCI for network devices.
		// Therefore, multifunction devices as well as additional bridges are not scanned.
		// We also limit scanning to the first 32 buses.
		let pci_config = PciConfigRegion::new();
		for bus in 0..PCI_MAX_BUS_NUMBER {
			for device in 0..PCI_MAX_DEVICE_NUMBER {
				let pci_address = PciAddress::new(0, bus, device, 0);
				let header = PciHeader::new(pci_address);

				let (vendor_id, device_id) = header.id(&pci_config);
				if device_id != u16::MAX && vendor_id != u16::MAX {
                    let addr = (pci_address.bus() as u32) << 16 | (pci_address.device() as u32) << 11;
					info!("Addr: {:#x}", addr);
                    let endpoint = EndpointHeader::from_header(header, &pci_config).unwrap();
                    let (_pin, line) = endpoint.interrupt(&pci_config);

					info!("Device ID: {:#x}  Vendor ID: {:#x}", device_id, vendor_id);

					if vendor_id == 0x10ec && (0x8138..=0x8139).contains(&device_id) {
						info!("Network card found.");
						let net_node = fdt.begin_node(format!("virtio_net@{:x}", addr).as_str())?;

						fdt.property_string("compatible", "realtek,rtl8139")?;
						fdt.property_u32("vendor-id", vendor_id as u32)?;
						fdt.property_u32("device-id", device_id as u32)?;
						fdt.property_u32("interrupts", line as u32)?;
						fdt.property_u32("pci-address", addr)?;
						fdt.property_array_u32("reg", &[addr, 0, 0, 0, 0, (0x02000010 | addr), 0, 0, 0, 0x100, (0x01000014 | addr), 0, 0, 0, 0x100])?;

						let mut assigned_addresses: Vec<u32> = Vec::new();
						for i in 0..MAX_BARS {
							if let Some(bar) = endpoint.bar(i.try_into().unwrap(), &pci_config) {
								match bar {
									Bar::Io { port } => {
										info!("BAR{:x} IO {{port: {:#X}}}", i, port);
										assigned_addresses.extend(alloc::vec![(0x81000014 | addr), 0, port, 0, 0x100]);
									}
									Bar::Memory32 { address , size, prefetchable } => {
										info!("BAR{:x} Memory32 {{address: {:#X}, size {:#X}, prefetchable: {:?}}}", i, address, size, prefetchable);
										assigned_addresses.extend(alloc::vec![(0x82000010 | addr), 0, address, 0, size]);
									}
									Bar::Memory64 { address , size, prefetchable } => {
										info!("BAR{:x} Memory64 {{address: {:#X}, size {:#X}, prefetchable: {:?}}}", i, address, size, prefetchable);
										assigned_addresses.extend(alloc::vec![(0x82000010 | addr), (address >> 32) as u32, address as u32, (size >> 32) as u32, size as u32]);
									}
								}
							}
						}
						fdt.property_array_u32("assigned-addresses", assigned_addresses.as_slice())?;

						fdt.end_node(net_node)?;
					}
				}
			}
		}

		fdt.end_node(pci_node)?;

		fdt.end_node(root_node)?;

		let fdt = fdt.finish()?;

		Ok(fdt.leak())
	}
}
