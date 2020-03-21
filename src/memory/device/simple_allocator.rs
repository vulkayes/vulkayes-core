use ash::vk;
use std::ops::Deref;

use crate::device::Device;
use crate::Vrc;

#[derive(Debug)]
pub struct SimpleDeviceMemoryAllocator {
	device: Vrc<Device>
}
impl SimpleDeviceMemoryAllocator {
	pub fn new(device: Vrc<Device>) -> Self {
		SimpleDeviceMemoryAllocator {
			device
		}
	}

	pub fn allocate() -> SimpleDeviceMemoryAllocation {

	}
}

pub struct SimpleDeviceMemoryAllocation {
	memory: vk::DeviceMemory
}
impl Deref for SimpleDeviceMemoryAllocation {
	type Target = vk::DeviceMemory;

	fn deref(&self) -> &Self::Target {
		&self.memory
	}
}
impl super::DeviceMemoryAllocation for SimpleDeviceMemoryAllocation {
	fn device(&self) -> &Vrc<Device> {
		&self.device()
	}

	fn bind_offset(&self) -> u64 {
		0
	}
}