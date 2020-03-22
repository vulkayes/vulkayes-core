use std::ops::Deref;

use ash::vk;

use crate::{device::Device, Vrc};

pub mod never;

#[cfg(feature = "naive_device_allocator")]
pub mod naive;

/// Trait for memory allocations done with memory allocators.
///
/// Objects implementing this trait will be stored in `Buffer` and `Image` wrappers.
/// They should implement `Drop` to properly clean up the memory when the resource is dropped.
///
/// ### Safety
///
/// * `device` must return the device this memory was allocated with.
/// * `bind_offset` must return a valid offset to bind image to.
/// * The implementing type must `Deref` to valid `vk::DeviceMemory` handle.
pub unsafe trait DeviceMemoryAllocation:
	Deref<Target = vk::DeviceMemory> + std::fmt::Debug + crate::util::sync::VSendSync + 'static
{
	/// Returns device from which the memory was allocated.
	fn device(&self) -> &Vrc<Device>;

	/// Returns value for the `offset` parameter when binding the memory.
	fn bind_offset(&self) -> vk::DeviceSize;
}

/// Trait for image memory allocators.
///
/// ### Safety
///
/// * `allocate` must return type implementing `DeviceMemoryAllocation` that can be bound to the `vk::Image` passed in.
pub unsafe trait ImageMemoryAllocator: std::fmt::Debug {
	type AllocationRequirements: std::fmt::Debug;
	type Allocation: DeviceMemoryAllocation;
	type Error: std::error::Error;

	fn allocate(
		&self,
		image: vk::Image,
		requirements: Self::AllocationRequirements
	) -> Result<Self::Allocation, Self::Error>;
}
/// Trait for buffer memory allocators.
///
/// ### Safety
///
/// * `allocate` must return type implementing `DeviceMemoryAllocation` that can be bound to the `vk::Buffer` passed in.
pub unsafe trait BufferMemoryAllocator: std::fmt::Debug {
	type AllocationRequirements: std::fmt::Debug;
	type Allocation: DeviceMemoryAllocation;
	type Error: std::error::Error;

	fn allocate(
		&self,
		buffer: vk::Buffer,
		requirements: Self::AllocationRequirements
	) -> Result<Self::Allocation, Self::Error>;
}
