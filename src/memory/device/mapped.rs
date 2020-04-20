use std::{
	fmt,
	num::{NonZeroU64, NonZeroUsize},
	ops::Deref,
	ptr::NonNull
};

use ash::{version::DeviceV1_0, vk};

use crate::{device::Device, prelude::Vrc};

use super::{MapMemoryImpl, UnmapMemoryImpl};

pub(super) struct DeviceMemoryMapping {
	pub ptr: Option<NonNull<[u8]>>,

	pub map_impl: MapMemoryImpl,
	pub unmap_impl: UnmapMemoryImpl
}
impl DeviceMemoryMapping {
	pub fn map(
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

	pub fn unmap(
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
// Safe because a mutable reference is required to access any field of this object
unsafe impl Send for DeviceMemoryMapping {}
unsafe impl Sync for DeviceMemoryMapping {}
impl fmt::Debug for DeviceMemoryMapping {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("DeviceMemoryMapping")
			.field("ptr", &self.ptr)
			.field("map_impl", &(self.map_impl.deref() as *const _))
			.field("unmap_impl", &(self.unmap_impl.deref() as *const _))
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
	pub(super) bytes: &'a mut [u8],

	pub(super) device: &'a Vrc<Device>,
	pub(super) memory: vk::DeviceMemory,

	pub(super) bind_offset: vk::DeviceSize
}
impl<'a> DeviceMemoryMappingAccess<'a> {
	pub fn bytes_mut(&mut self) -> &mut [u8] {
		self.bytes
	}

	/// Write a slice of `T`s into this memory.
	///
	/// The `offset` parameter can be used to control the initial offset of the write in bytes.
	/// If the `offset` is greater than the length of `self.bytes_mut()` it will be clamped to `self.bytes_mut().len()` (and nothing will be written).
	///
	/// The `stride` parameter can be used to control the stride of the write in bytes.
	/// For example, to write `u32`s aligned to 8 bytes, `stride` can either be `SliceWriteStride::Align(8)`
	/// or `SliceWriteStride::Stride(8)`.
	///
	/// Note, however, that this can have an effect on the performance. The method will use:
	/// * `ptr::copy_nonoverlapping` if `stride.for_t::<T>() == SliceWriteStride::Implicit.for_t::<T>()`
	/// * `ptr::write` in a loop if `stride % std::mem::align_of::<T>() == 0` and `bytes.as_mut_ptr() as usize % std::mem::align_of::<T>() == 0`
	/// * `ptr::write_unaligned` in a loop otherwise
	///
	/// Number of `T`s written is the minimum of `data.len()` and `self.bytes()[offset..].len() / stride`.
	pub fn write_slice<T: Copy>(&mut self, data: &[T], offset: usize, stride: SliceWriteStride) {
		let bytes = self.bytes_mut();
		let offset = offset.min(bytes.len());

		let bytes = &mut bytes[offset ..];
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

	pub fn flush(&mut self) -> Result<(), FlushError> {
		let mapped_memory_range = vk::MappedMemoryRange::builder()
			.memory(self.memory)
			.offset(self.bind_offset)
			.size(self.size().get())
			.build();

		unsafe {
			self.device
				.flush_mapped_memory_ranges(&[mapped_memory_range])
				.map_err(Into::into)
		}
	}

	pub fn invalidate(&mut self) -> Result<(), FlushError> {
		let mapped_memory_range = vk::MappedMemoryRange::builder()
			.memory(self.memory)
			.offset(self.bind_offset)
			.size(self.size().get())
			.build();

		unsafe {
			self.device
				.invalidate_mapped_memory_ranges(&[mapped_memory_range])
				.map_err(Into::into)
		}
	}

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
		unsafe { NonZeroU64::new_unchecked(self.bytes.len() as u64) }
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

vk_result_error! {
	#[derive(Debug)]
	pub enum FlushError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}
