use std::{fmt, num::NonZeroU64, ops::Deref};

use ash::vk;

use crate::{
	device::Device,
	memory::{
		device::{allocator::BufferMemoryAllocator, DeviceMemoryAllocation},
		host::HostMemoryAllocator
	},
	prelude::Vrc,
	queue::sharing_mode::SharingMode
};

use super::{error, params};

pub struct Buffer {
	device: Vrc<Device>,
	buffer: vk::Buffer,
	memory: Option<DeviceMemoryAllocation>,

	usage: vk::BufferUsageFlags,
	size: NonZeroU64,

	// TODO: Sharing mode + indices?
	host_memory_allocator: HostMemoryAllocator
}
impl Buffer {
	pub fn new<A: BufferMemoryAllocator>(
		device: Vrc<Device>,
		size: NonZeroU64,
		usage: vk::BufferUsageFlags,
		sharing_mode: SharingMode<impl AsRef<[u32]>>,
		allocator_params: params::BufferAllocatorParams<A>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::BufferError<A::Error>> {
		#[cfg(feature = "runtime_implicit_validations")]
		{
			if usage.is_empty() {
				return Err(error::BufferError::UsageEmpty)
			}
		}

		let create_info = vk::BufferCreateInfo::builder()
			.size(size.get())
			.usage(usage)
			.sharing_mode(sharing_mode.sharing_mode())
			.queue_family_indices(sharing_mode.indices());

		unsafe {
			Self::from_create_info(device, create_info, allocator_params, host_memory_allocator)
		}
	}

	/// Creates a new `Buffer` from existing `BufferCreateInfo`
	///
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateBuffer.html>.
	pub unsafe fn from_create_info<A: BufferMemoryAllocator>(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::BufferCreateInfo>,
		allocator_params: params::BufferAllocatorParams<A>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::BufferError<A::Error>> {
		let c_info = create_info.deref();

		log_trace_common!(
			"Create buffer:",
			device,
			c_info,
			allocator_params,
			host_memory_allocator
		);
		let buffer = device.create_buffer(c_info, host_memory_allocator.as_ref())?;

		let memory = match allocator_params {
			params::BufferAllocatorParams::Some {
				allocator,
				requirements
			} => {
				let memory = allocator
					.allocate(buffer, requirements)
					.map_err(error::BufferError::AllocationError)?;

				#[cfg(feature = "runtime_implicit_validations")]
				{
					if memory.device() != &device {
						return Err(error::BufferError::MemoryDeviceMismatch)
					}
				}

				// TODO: Error here leaks buffer
				device.bind_buffer_memory(buffer, *memory.deref(), memory.bind_offset())?;
				Some(memory)
			}
			params::BufferAllocatorParams::None => None
		};

		let size = NonZeroU64::new_unchecked(create_info.size);

		Ok(Vrc::new(Buffer {
			device,
			buffer,
			memory,

			usage: c_info.usage,
			size,

			host_memory_allocator
		}))
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}

	pub const fn usage(&self) -> vk::BufferUsageFlags {
		self.usage
	}

	pub const fn size(&self) -> NonZeroU64 {
		self.size
	}

	/// Returns the length of this buffer in number of `T`s.
	pub fn size_of<T>(&self) -> usize {
		self.size().get() as usize / std::mem::size_of::<T>()
	}

	// TODO: Cannot be const because of Sized
	pub fn memory(&self) -> Option<&DeviceMemoryAllocation> {
		self.memory.as_ref()
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::Buffer>, Deref, Borrow, Eq, Hash, Ord for Buffer {
		target = { buffer }
	}
}
impl Drop for Buffer {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.device
				.destroy_buffer(self.buffer, self.host_memory_allocator.as_ref());
		}
	}
}
impl fmt::Debug for Buffer {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Buffer")
			.field("device", &self.device)
			.field("buffer", &crate::util::fmt::format_handle(self.buffer))
			.field(
				"memory",
				&self
					.memory
					.as_ref()
					.map(|m| crate::util::fmt::format_handle(*m.deref().deref()))
			)
			.field("usage", &self.usage)
			.field("size", &self.size)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
