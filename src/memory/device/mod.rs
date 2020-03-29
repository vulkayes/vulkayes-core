use std::{
	fmt,
	num::{NonZeroU64, NonZeroUsize},
	ops::Deref,
	ptr::NonNull
};

use ash::vk;

use crate::{device::Device, util::sync::Vutex, Vrc};

pub mod allocator;

#[cfg(feature = "naive_device_allocator")]
pub mod naive;
pub mod never;

struct DeviceMemoryMapping {
	ptr: Option<NonNull<[u8]>>,

	map_impl: Box<
		dyn FnMut(
			&Vrc<Device>,
			vk::DeviceMemory,
			vk::DeviceSize,
			NonZeroU64
		) -> Result<NonNull<[u8]>, MapError>
	>,
	unmap_impl:
		Box<dyn FnMut(&Vrc<Device>, vk::DeviceMemory, vk::DeviceSize, NonZeroU64, NonNull<[u8]>)>
}
impl DeviceMemoryMapping {
	fn map(
		&mut self,
		device: &Vrc<Device>,
		memory: vk::DeviceMemory,
		bind_offset: vk::DeviceSize,
		size: NonZeroU64
	) -> Result<(), MapError> {
		log_trace_common!("Mapping memory:", self);
		let ptr = (self.map_impl)(device, memory, bind_offset, size)?;

		self.ptr = Some(ptr);

		Ok(())
	}

	fn unmap(
		&mut self,
		device: &Vrc<Device>,
		memory: vk::DeviceMemory,
		bind_offset: vk::DeviceSize,
		size: NonZeroU64
	) -> bool {
		log_trace_common!("Unmapping memory:", self);
		match self.ptr.take() {
			None => false,
			Some(ptr) => {
				(self.unmap_impl)(device, memory, bind_offset, size, ptr);

				true
			}
		}
	}
}
impl fmt::Debug for DeviceMemoryMapping {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("DeviceMemoryMapping")
			.field("ptr", &self.ptr)
			.field("map_impl", &(self.map_impl.deref() as *const _))
			.field("unmap_impl", &(self.unmap_impl.deref() as *const _))
			.finish()
	}
}

/// Struct that represents a device memory allocation.
pub struct DeviceMemoryAllocation {
	device: Vrc<Device>,
	memory: vk::DeviceMemory,

	bind_offset: vk::DeviceSize,
	size: NonZeroU64,

	mapping: Vutex<DeviceMemoryMapping>,

	/// This is a drop function that will be called when this memory allocation is dropped.
	/// Wrapped in `Option` because it is moved out in `Drop`.
	drop_impl: Option<Box<dyn FnOnce(&Vrc<Device>, vk::DeviceMemory, vk::DeviceSize, NonZeroU64)>>
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

		map_impl: Box<
			dyn FnMut(
				&Vrc<Device>,
				vk::DeviceMemory,
				vk::DeviceSize,
				NonZeroU64
			) -> Result<NonNull<[u8]>, MapError>
		>,
		unmap_impl: Box<
			dyn FnMut(&Vrc<Device>, vk::DeviceMemory, vk::DeviceSize, NonZeroU64, NonNull<[u8]>)
		>,

		drop_impl: Box<dyn FnOnce(&Vrc<Device>, vk::DeviceMemory, vk::DeviceSize, NonZeroU64)>
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
	pub fn map_access(
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

			bind_offset: self.bind_offset,
			size: self.size
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

#[derive(Copy, Clone, Debug)]
pub enum MappingAccessResult {
	/// Continue on without doing anything.
	Continue,
	/// Unmap memory before returning.
	Unmap
}

#[derive(Copy, Clone, Debug)]
pub enum SliceWriteStride {
	/// Use the stride that is implicit to the type.
	///
	/// This allows optimizing the copy to `copy_nonoverlapping`.
	Implicit,
	/// Align the written values to this alignment.
	///
	/// If the value is not a power of two, this is treated as `Implicit`.
	Align(NonZeroUsize),
	/// Use custom stride.
	///
	/// Values will be manually copied in loop at this stride. If the value is smaller than
	/// the size of the type, that size is used instead.
	Stride(NonZeroUsize)
}
impl SliceWriteStride {
	pub fn for_t<T>(&self) -> usize {
		match self {
			SliceWriteStride::Implicit => {
				crate::util::align_up(std::mem::size_of::<T>(), std::mem::align_of::<T>())
			}
			SliceWriteStride::Align(align) => {
				crate::util::align_up(std::mem::size_of::<T>(), align.get())
			}
			SliceWriteStride::Stride(stride) => stride.get().max(std::mem::size_of::<T>())
		}
	}
}
impl Default for SliceWriteStride {
	fn default() -> Self {
		SliceWriteStride::Implicit
	}
}

#[derive(Debug)]
pub struct DeviceMemoryMappingAccess<'a> {
	bytes: &'a mut [u8],

	device: &'a Vrc<Device>,
	memory: vk::DeviceMemory,

	bind_offset: vk::DeviceSize,
	size: NonZeroU64
}
impl<'a> DeviceMemoryMappingAccess<'a> {
	pub fn bytes_mut(&mut self) -> &mut [u8] {
		self.bytes
	}

	/// Write a slice of `T`s into this memory.
	///
	/// The `stride` parameter can be used to control the stride of the write in bytes.
	/// For example, to write `u32`s aligned to 8 bytes, `stride` can either be `SliceWriteStride::Align(8)`
	/// or `SliceWriteStride::Stride(8)`.
	///
	/// Note, however, that this can have an effect on the performance. The method will use:
	/// * `ptr::copy_nonoverlapping` if `stride.for_t::<T>() == SliceWriteStride::Implicit.for_t::<T>()`
	/// * `ptr::write` in a loop if `stride % std::mem::align_of::<T>() == 0` and `self.bytes.as_mut_ptr() as usize % std::mem::align_of::<T>() == 0`
	/// * `ptr::write_unaligned` in a loop otherwise
	///
	/// Number of `T`s written is the minimum of `data.len()` and `self.bytes().len() / stride`.
	pub fn write_slice<T: Copy>(&mut self, data: &[T], stride: SliceWriteStride) {
		let bytes = self.bytes_mut();
		let stride = stride.for_t::<T>();
		let count = data.len().min(bytes.len() / stride);

		log_trace_common!(
			"Writing slice to mapped memory:",
			bytes.as_ptr(),
			stride,
			count
		);

		if stride == SliceWriteStride::Implicit.for_t::<T>() {
			// This can be done using copy_nonoverlapping because the stride is the implicit stride
			// It also doesn't matter here that the destination pointer might be unaligned because copy_nonoverlapping internally works with bytes.
			unsafe {
				std::ptr::copy_nonoverlapping(data.as_ptr(), bytes.as_mut_ptr() as *mut T, count);
			}
		} else if stride % std::mem::align_of::<T>() == 0
			&& bytes.as_mut_ptr() as usize % std::mem::align_of::<T>() == 0
		{
			// If stride is not the same as the implicit stride, then this will have to be a manual loop
			// But if both the stride and destination pointer are aligned, then we can use aligned writes
			for index in 0 .. count {
				unsafe {
					std::ptr::write(
						bytes.as_mut_ptr().offset((index * stride) as isize) as *mut T,
						data[index]
					);
				}
			}
		} else {
			// In the worst case, we have to use write_unaligned
			for index in 0 .. count {
				unsafe {
					std::ptr::write_unaligned(
						bytes.as_mut_ptr().offset((index * stride) as isize) as *mut T,
						data[index]
					);
				}
			}
		}
	}

	// TODO: Flush and invalidate?

	pub const fn device(&self) -> &Vrc<Device> {
		self.device
	}

	pub const fn memory(&self) -> vk::DeviceMemory {
		self.memory
	}

	pub const fn bind_offset(&self) -> vk::DeviceSize {
		self.bind_offset
	}

	pub const fn size(&self) -> NonZeroU64 {
		self.size
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum MapError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_MEMORY_MAP_FAILED
		}
	}
}
