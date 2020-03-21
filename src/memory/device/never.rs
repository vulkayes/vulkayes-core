use std::ops::Deref;

use ash::vk;

use crate::{device::Device, Vrc};

use super::{BufferMemoryAllocator, DeviceMemoryAllocation, ImageMemoryAllocator};

/// Device memory allocator that is statically impossible.
///
/// This device allocator along with its `NeverMemoryAllocation` can be used for images and buffers
/// that should not have any memory bound through the provided mechanism.
#[derive(Debug)]
pub enum NeverDeviceAllocator {}
unsafe impl ImageMemoryAllocator for NeverDeviceAllocator {
	type Allocation = NeverMemoryAllocation;
	type Error = std::convert::Infallible;

	// TODO: Replace with never `!` type when stable

	fn allocate(&mut self, _: vk::Image) -> Result<Self::Allocation, Self::Error> {
		unreachable!()
	}
}
unsafe impl BufferMemoryAllocator for NeverDeviceAllocator {
	type Allocation = NeverMemoryAllocation;
	type Error = std::convert::Infallible;

	// TODO: Replace with never `!` type when stable

	fn allocate(&mut self, _: vk::Buffer) -> Result<Self::Allocation, Self::Error> {
		unreachable!()
	}
}

/// Device memory allocation that is statically impossible.
///
/// This is the return type of the `NeverDeviceAllocator::allocate` methods.
#[derive(Debug)]
pub enum NeverMemoryAllocation {}
impl Deref for NeverMemoryAllocation {
	type Target = vk::DeviceMemory;

	fn deref(&self) -> &Self::Target {
		unreachable!()
	}
}
unsafe impl DeviceMemoryAllocation for NeverMemoryAllocation {
	fn device(&self) -> &Vrc<Device> {
		unreachable!()
	}

	fn bind_offset(&self) -> u64 {
		unreachable!()
	}
}
