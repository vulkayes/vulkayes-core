use ash::vk;

use super::{
	allocator::{BufferMemoryAllocator, ImageMemoryAllocator},
	DeviceMemoryAllocation
};

/// Device memory allocator that is statically impossible.
///
/// This device allocator along with its `NeverMemoryAllocation` can be used for images and buffers
/// that should not have any memory bound through the provided mechanism.
#[derive(Debug)]
pub enum NeverDeviceAllocator {}
unsafe impl ImageMemoryAllocator for NeverDeviceAllocator {
	type AllocationRequirements = std::convert::Infallible;
	type Error = std::convert::Infallible;

	// TODO: Replace with never `!` type when stable

	fn allocate(&self, _: vk::Image, _: Self::AllocationRequirements) -> Result<DeviceMemoryAllocation, Self::Error> {
		unreachable!()
	}
}
unsafe impl BufferMemoryAllocator for NeverDeviceAllocator {
	type AllocationRequirements = std::convert::Infallible;
	type Error = std::convert::Infallible;

	// TODO: Replace with never `!` type when stable

	fn allocate(&self, _: vk::Buffer, _: Self::AllocationRequirements) -> Result<DeviceMemoryAllocation, Self::Error> {
		unreachable!()
	}
}
