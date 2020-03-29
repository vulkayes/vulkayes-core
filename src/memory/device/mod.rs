use std::ops::Deref;
use std::num::NonZeroU64;

use ash::vk;

use crate::{device::Device, Vrc};
use std::ffi::c_void;

pub mod never;
// #[cfg(feature = "naive_device_allocator")]
// pub mod naive;

/// Trait for memory allocations done with memory allocators.
///
/// Objects implementing this trait will be stored in `Buffer` and `Image` wrappers.
/// They should implement `Drop` to properly clean up the memory when the resource is dropped.
///
/// ### Safety
///
/// * `device` must return the device this memory was allocated with.
/// * `size` must return the size of the memory sub-allocation this object refers to (not necessarily the whole `vk::DeviceMemory` size).
/// * `bind_offset` must return a valid offset to bind image to.
/// * `map` must be internally synchronized and must return an error if the same `vk::DeviceMemory` object is attempted to be mapped more than once at a time.
/// * The implementing type must `Deref` to valid `vk::DeviceMemory` handle.
// pub unsafe trait DeviceMemoryAllocation:
// 	Deref<Target = vk::DeviceMemory> + crate::util::sync::VSendSync + std::any::Any + std::fmt::Debug + 'static
// {
// 	// type Mapping: DeviceMemoryAllocationMapping;
// 	// type MappingError: std::error::Error;
//
// 	/// Returns device from which the memory was allocated.
// 	fn device(&self) -> &Vrc<Device>;
//
// 	/// Returns the size of this sub-allocation.
// 	fn size(&self) -> NonZeroU64;
//
// 	/// Returns value for the `offset` parameter when binding the memory.
// 	fn bind_offset(&self) -> vk::DeviceSize;
//
// 	// /// Attempts to map this memory, returning an error if not possible.
// 	// fn map(&self) -> Result<Self::MappedMemory, Self::MappingError>;
// }

pub struct MappedDeviceMemory {
	ptr: *mut [u8],
	/// This is a drop function that will be when this mapping is dropped.
	drop_impl: Box<dyn FnOnce(&Vrc<Device>, vk::DeviceMemory, NonZeroU64, vk::DeviceSize)>
}
impl MappedDeviceMemory {
	///
	///
	/// ### Safety
	///
	/// * `memory` must be
	pub unsafe fn new(
		memory: *mut [u8],
		drop_impl: Box<dyn FnOnce(&Vrc<Device>, vk::DeviceMemory, NonZeroU64, vk::DeviceSize)>
	) -> Self {

	}
}

#[derive(Debug, Copy, Clone)]
pub struct MappedDeviceMemoryAccess<'a> {
	memory: &'a mut [u8],

	device: &'a Vrc<Device>,
	bind_offset: vk::DeviceSize,
}
impl<'a> MappedDeviceMemoryAccess<'a> {
	pub const fn memory(self) -> &'a mut [u8] {
		self.memory
	}

	/// Write a slice of `T`s into this memory.
	///
	/// The `stride` parameter can be used to control the stride of the write in bytes.
	/// For example, to write 3 `u32`s aligned to 8 bytes, set the `stride` to `Some(8)`.
	///
	/// Setting `stride` to `None` or less than or equal to `size_of::<T>()` will default
	/// to the implicit stride of `size_of::<T>()` and use `copy_nonoverlapping` instead of a loop of `write`s.
	///
	/// Number of `T`s written is the minimum of `data.len()` and `self.memory().len() / stride.unwrap_or(size_of::<T>())`.
	pub fn write_slice<T: Copy>(self, data: &[T], stride: Option<vk::DeviceSize>) {
		let t_size = std::mem::size_of::<T>();

		// Resolve the case when stride is less than size of the data.
		let stride = match stride {
			Some(stride) if stride > t_size as u64 => Some(stride),
			_ => None
		};

		let bytes = self.memory();
		// Compute count of Ts that will be copied
		let count = data.len().min(bytes.len() / stride.unwrap_or(t_size));

		log_trace_common!("Writing to mapped memory:", bytes.as_ptr(), count, stride);
		if let Some(stride) = stride {
			// If stride is set, then this will have to be a manual loop
			for index in 0 .. count {
				unsafe {
					std::ptr::write(
						bytes.as_mut_ptr().offset(index as isize * stride as isize) as *mut T,
						data[index]
					);
				}
			}
		} else {
			// With stride being unset, the implicit stride that is the size of T is used.
			unsafe {
				std::ptr::copy_nonoverlapping(
					data.as_ptr() as *const u8,
					bytes.as_mut_ptr(),
					count * t_size
				);
			}
		}
	}
}

/// Struct that represents a device memory allocation.
///
/// This struct
pub struct DeviceMemoryAllocation {
	device: Vrc<Device>,
	memory: vk::DeviceMemory,

	size: NonZeroU64,
	bind_offset: vk::DeviceSize,

	// mapping: Vutex<Option<MappedDeviceMemory>>, // TODO

	/// This is a drop function that will be called when this memory allocation is dropped.
	drop_impl: Box<dyn FnOnce(&Vrc<Device>, vk::DeviceMemory, NonZeroU64, vk::DeviceSize)>
}
impl DeviceMemoryAllocation {
	/// Creates a new memory allocation from parameters.
	///
	/// The `drop_impl` parameter is a `FnOnce` that is called in the `Drop` implementation of this struct.
	/// It should properly clean up the allocation according to the allocator implementation.
	///
	/// ### Safety
	///
	/// * `memory` must have been allocated from the `device`.
	/// * `bind_offset + size` must be less than or equal to the size of the entire `vk::DeviceMemory` allocation
	pub unsafe fn new(
		device: Vrc<Device>,
		memory: vk::DeviceMemory,
		size: NonZeroU64,
		bind_offset: vk::DeviceSize,
		drop_impl: Box<dyn FnOnce(&Vrc<Device>, vk::DeviceMemory, NonZeroU64, vk::DeviceSize)>
	) -> Self {
		DeviceMemoryAllocation {
			device,
			memory,
			size,
			bind_offset,

			drop_impl
		}
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}

	pub const fn size(&self) -> NonZeroU64 {
		self.size
	}

	pub const fn bind_offset(&self) -> vk::DeviceSize {
		self.bind_offset
	}

	/// Returns true if this memory is currently mapped.
	///
	/// Note that this check requires locking a `Vutex`.
	///
	/// ### Panic
	///
	/// This function will panic if the `Vutex` is poisoned.
	pub fn is_mapped(&self) -> bool {
		let lock = self.mapping.lock().expect("vutex poisoned");

		return lock.is_some()
	}

	///
	pub fn map_access(&self, accessor: impl FnOnce(MappedDeviceMemoryAccess) -> bool) {

	}
}
impl Drop for DeviceMemoryAllocation {
	fn drop(&mut self) {
		// Drop mapping, if any
		self.mapping.take();

		self.drop_impl(
			&self.device,
			self.memory,
			self.size,
			self.bind_offset
		)
	}
}

/// Trait for image memory allocators.
///
/// ### Safety
///
/// * `allocate` must return a valid `DeviceMemoryAllocation` that can be bound to the `vk::Image` passed in.
pub unsafe trait ImageMemoryAllocator: std::fmt::Debug {
	type AllocationRequirements: std::fmt::Debug;
	type Error: std::error::Error;

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
	type Error: std::error::Error;

	fn allocate(
		&self,
		buffer: vk::Buffer,
		requirements: Self::AllocationRequirements
	) -> Result<DeviceMemoryAllocation, Self::Error>;
}
