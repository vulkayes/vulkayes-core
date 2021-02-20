use ash::vk;

use super::DeviceMemoryAllocation;

/// Trait for image memory allocators.
///
/// ### Safety
///
/// * `allocate` must return a valid `DeviceMemoryAllocation` that can be bound to the `vk::Image` passed in.
pub unsafe trait ImageMemoryAllocator: std::fmt::Debug {
	type AllocationRequirements: std::fmt::Debug;
	type Error: std::error::Error + 'static;

	fn allocate(
		&self,
		image: vk::Image,
		requirements: Self::AllocationRequirements
	) -> Result<DeviceMemoryAllocation, Self::Error>;
}
/// Trait for buffer memory allocators.
///
/// ### Safety
///
/// * `allocate` must return a valid `DeviceMemoryAllocation` that can be bound to the `vk::Buffer` passed in.
pub unsafe trait BufferMemoryAllocator: std::fmt::Debug {
	type AllocationRequirements: std::fmt::Debug;
	type Error: std::error::Error + 'static;

	fn allocate(
		&self,
		buffer: vk::Buffer,
		requirements: Self::AllocationRequirements
	) -> Result<DeviceMemoryAllocation, Self::Error>;
}
