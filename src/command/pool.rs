use std::{fmt::Debug, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::device::Device;
use crate::{memory::host::HostMemoryAllocator, queue::Queue, util::sync::Vutex, Vrc};

/// Internally synchronized command pool.
pub struct CommandPool {
	device: Vrc<Device>,
	queue_family_index: u32,

	pool: Vutex<vk::CommandPool>,

	host_memory_allocator: HostMemoryAllocator
}
impl CommandPool {
	/// Note: `PROTECTED` flag value is currently ignored.
	pub fn new(
		queue: &Vrc<Queue>,
		flags: vk::CommandPoolCreateFlags,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, CommandPoolError> {
		let flags = flags - vk::CommandPoolCreateFlags::PROTECTED;

		let create_info = vk::CommandPoolCreateInfo::builder()
			.flags(flags)
			.queue_family_index(queue.queue_family_index());

		unsafe { Self::from_create_info(queue, create_info, host_memory_allocator) }
	}

	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateCommandPool.html>
	pub unsafe fn from_create_info(
		queue: &Vrc<Queue>,
		create_info: impl Deref<Target = vk::CommandPoolCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, CommandPoolError> {
		log_trace_common!(
			"Creating command pool:",
			queue,
			create_info.deref(),
			host_memory_allocator
		);
		let pool = queue
			.device()
			.create_command_pool(create_info.deref(), host_memory_allocator.as_ref())?;

		Ok(Vrc::new(CommandPool {
			device: queue.device().clone(),
			queue_family_index: queue.queue_family_index(),

			pool: Vutex::new(pool),
			host_memory_allocator
		}))
	}

	/// ### Panic
	///
	/// This function will panic if the pool `Vutex` is poisoned.
	pub fn allocate_command_buffers(
		&self,
		level: vk::CommandBufferLevel,
		count: std::num::NonZeroU32
	) -> Result<Vec<vk::CommandBuffer>, super::buffer::CommandBufferError> {
		let lock = self.pool.lock().expect("vutex poisoned");

		let alloc_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(*lock)
			.level(level)
			.command_buffer_count(count.get());

		log_trace_common!(
			"Allocating command buffers:",
			crate::util::fmt::format_handle(*lock),
			alloc_info.deref()
		);

		unsafe {
			self.device
				.allocate_command_buffers(alloc_info.deref())
				.map_err(Into::into)
		}
	}

	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkFreeCommandBuffers.html>
	///
	/// ### Panic
	///
	/// This function will panic if the pool `Vutex` is poisoned.
	pub unsafe fn free_command_buffers(&self, buffers: impl AsRef<[vk::CommandBuffer]>) {
		let lock = self.pool.lock().expect("vutex poisoned");

		log_trace_common!(
			"Freeing command buffers:",
			crate::util::fmt::format_handle(*lock),
			buffers.as_ref()
		);

		self.device
			.free_command_buffers(*lock, buffers.as_ref())
	}

	pub const fn queue_family_index(&self) -> u32 {
		self.queue_family_index
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for CommandPool {
		type Target = Vutex<ash::vk::CommandPool> { pool }

		to_handle { .lock().expect("vutex poisoned").deref() }
	}
}
impl Drop for CommandPool {
	fn drop(&mut self) {
		let lock = self.pool.lock().expect("vutex poisoned");

		unsafe {
			self.device.destroy_command_pool(*lock, self.host_memory_allocator.as_ref())
		}
	}
}
impl Debug for CommandPool {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct("CommandPool")
			.field("device", &self.device)
			.field("queue_family_index", &self.queue_family_index)
			.field("pool", &self.pool)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum CommandPoolError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}
