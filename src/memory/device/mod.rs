use std::{fmt, num::NonZeroU64, ops::Deref, ptr::NonNull};

use ash::vk;

use mapped::DeviceMemoryMapping;
pub use mapped::{DeviceMemoryMappingAccess, MapError, MappingAccessResult, SliceWriteStride};

use crate::{device::Device, util::sync::Vutex, prelude::Vrc};

pub mod allocator;
mod mapped;

#[cfg(feature = "naive_device_allocator")]
pub mod naive;
pub mod never;

type DropAllocImpl =
	Box<VSendSync![dyn FnOnce(&Vrc<Device>, vk::DeviceMemory, vk::DeviceSize, NonZeroU64)]>;
type MapMemoryImpl = Box<
	VSendSync![
		dyn FnMut(
			&Vrc<Device>,
			vk::DeviceMemory,
			vk::DeviceSize,
			NonZeroU64
		) -> Result<NonNull<[u8]>, MapError>
	]
>;
type UnmapMemoryImpl = Box<
	VSendSync![
		dyn FnMut(&Vrc<Device>, vk::DeviceMemory, vk::DeviceSize, NonZeroU64, NonNull<[u8]>)
	]
>;

/// Struct that represents a device memory allocation.
pub struct DeviceMemoryAllocation {
	device: Vrc<Device>,
	memory: vk::DeviceMemory,

	bind_offset: vk::DeviceSize,
	size: NonZeroU64,

	mapping: Vutex<DeviceMemoryMapping>,

	/// This is a drop function that will be called when this memory allocation is dropped.
	/// Wrapped in `Option` because it is moved out in `Drop`.
	drop_impl: Option<DropAllocImpl>
}
impl DeviceMemoryAllocation {
	/// Creates a new memory allocation from parameters.
	///
	/// The `map_impl` parameter is a `FnMut` that is called when the memory is to be mapped. The memory mapping
	/// is stored inside the allocation `Vutex` until it is manually unmapped. Note that it is up to the user to
	/// ensure that the memory is actually mappable.
	///
	/// The `unmap_impl` parameter is a `FnMut` that is called when the memory is to be unmapped. It is guaranteed to be
	/// called with the same parameters as the corresponding `map_impl` and the pointer returned from the corresponding `map_impl`.
	///
	/// The `drop_impl` parameter is a `FnOnce` that is called in the `Drop` implementation of this struct.
	/// It should properly clean up the allocation according to the allocator implementation.
	///
	/// ### Safety
	///
	/// * `memory` must have been allocated from the `device`.
	/// * `bind_offset + size` must be less than or equal to the size of the entire `vk::DeviceMemory` allocation
	/// * `map_impl(device, memory, size, offset)` must return a valid `NonNull<u8>` that is a mapping of `memory` range starting at `offset` with `size`.
	/// * `map_impl` must return an error if the memory object is already mapped
	pub unsafe fn new(
		device: Vrc<Device>,
		memory: vk::DeviceMemory,
		bind_offset: vk::DeviceSize,
		size: NonZeroU64,

		map_impl: MapMemoryImpl,
		unmap_impl: UnmapMemoryImpl,

		drop_impl: DropAllocImpl
	) -> Self {
		DeviceMemoryAllocation {
			device,
			memory,
			bind_offset,
			size,

			mapping: Vutex::new(DeviceMemoryMapping {
				ptr: None,
				map_impl,
				unmap_impl
			}),

			drop_impl: Some(drop_impl)
		}
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}

	pub const fn bind_offset(&self) -> vk::DeviceSize {
		self.bind_offset
	}

	pub const fn size(&self) -> NonZeroU64 {
		self.size
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

		return lock.ptr.is_some()
	}

	/// Unmaps the memory if it is currently mapped.
	///
	/// Returns whether the memory was mapped.
	///
	/// ### Panic
	///
	/// This function will panic if the `Vutex` is poisoned.
	pub fn unmap(&self) -> bool {
		let mut lock = self.mapping.lock().expect("vutex poisoned");

		lock.unmap(&self.device, self.memory, self.bind_offset, self.size)
	}

	/// Provides mutable access to the mapped memory, possibly mapping it in the process.
	///
	/// The `accessor` parameter receives an access object into the mapped memory. If it returns `false`, the memory will be unmapped before returning.
	///
	/// ### Panic
	///
	/// This function will panic if the `Vutex` is poisoned.
	pub fn map_memory_with(
		&self,
		accessor: impl FnOnce(DeviceMemoryMappingAccess) -> MappingAccessResult
	) -> Result<(), MapError> {
		let mut lock = self.mapping.lock().expect("vutex poisoned");

		if let None = lock.ptr {
			lock.map(&self.device, self.memory, self.bind_offset, self.size)?;
		}

		// SAFETY: We are under a Vutex, which
		let bytes = unsafe { lock.ptr.as_mut().unwrap().as_mut() };
		let access = DeviceMemoryMappingAccess {
			bytes,
			device: &self.device,
			memory: self.memory,

			bind_offset: self.bind_offset // size: self.size
		};

		let result = accessor(access);
		match result {
			MappingAccessResult::Continue => (),
			MappingAccessResult::Unmap => {
				lock.unmap(&self.device, self.memory, self.bind_offset, self.size);
			}
		}

		Ok(())
	}
}
impl Deref for DeviceMemoryAllocation {
	type Target = vk::DeviceMemory;

	fn deref(&self) -> &Self::Target {
		&self.memory
	}
}
impl Drop for DeviceMemoryAllocation {
	fn drop(&mut self) {
		let mut lock = self.mapping.lock().expect("vutex poisoned");
		log_trace_common!("Dropping", self, lock);

		if lock.ptr.is_some() {
			lock.unmap(&self.device, self.memory, self.bind_offset, self.size);
		}

		(self.drop_impl.take().unwrap())(&self.device, self.memory, self.bind_offset, self.size)
	}
}
impl fmt::Debug for DeviceMemoryAllocation {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("DeviceMemoryAllocation")
			.field("device", &self.device)
			.field("memory", &crate::util::fmt::format_handle(self.memory))
			.field("bind_offset", &self.bind_offset)
			.field("size", &self.size)
			.field("mapping", &self.mapping)
			.field(
				"drop_impl",
				&self.drop_impl.as_ref().map(|b| b.as_ref() as *const _)
			)
			.finish()
	}
}
