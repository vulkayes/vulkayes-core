use std::ops::Deref;
use std::fmt::Debug;

use ash::vk;
use ash::version::DeviceV1_0;

use crate::Vrc;
use crate::util::sync::Vutex;
use crate::queue::Queue;
use crate::memory::host::HostMemoryAllocator;

/// Internally synchronized command pool.
pub struct CommandPool {
	queue: Vrc<Queue>,
	pool: Vutex<vk::CommandPool>,

	allocation_callbacks: Option<vk::AllocationCallbacks>
}
impl CommandPool {
	///
	///
	/// Note: `PROTECTED` flag value is currently ignored.
	pub fn new(
		queue: Vrc<Queue>,
		flags: vk::CommandPoolCreateFlags,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, CommandPoolError> {
		let flags = flags - vk::CommandPoolCreateFlags::PROTECTED;

		let create_info = vk::CommandPoolCreateInfo::builder()
			.flags(flags)
			.queue_family_index(queue.queue_family_index());

		unsafe {
			Self::from_create_info(
				queue,
				create_info,
				host_memory_allocator
			)
		}
	}

	pub unsafe fn from_create_info(
		queue: Vrc<Queue>,
		create_info: impl Deref<Target = vk::CommandPoolCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, CommandPoolError> {
		let allocation_callbacks: Option<vk::AllocationCallbacks> = host_memory_allocator.into();

		log::debug!(
			"Creating command pool with {:#?} {:#?} {:#?}",
			queue,
			create_info.deref(),
			allocation_callbacks
		);
		let pool = queue.device().create_command_pool(
			create_info.deref(),
			allocation_callbacks.as_ref()
		)?;

		Ok(
			Vrc::new(
				CommandPool {
					queue,
					pool: Vutex::new(pool),
					allocation_callbacks
				}
			)
		)
	}

	pub unsafe fn allocate_command_buffers(
		&self,
		level: vk::CommandBufferLevel,
		count: std::num::NonZeroU32
	) -> Result<Vec<vk::CommandBuffer>, super::buffer::CommandBufferError> {
		let lock = self.pool.lock().expect("mutex poisoned");

		let alloc_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(*lock)
			.level(level)
			.command_buffer_count(count.get())
		;

		log::trace!(
			"Allocating command buffers with {:#?} {:#?}",
			self,
			alloc_info.deref()
		);

		self.queue.device().allocate_command_buffers(
			alloc_info.deref()
		).map_err(|e| e.into())
	}

	pub unsafe fn free_command_buffers(
		&self,
		buffers: impl AsRef<[vk::CommandBuffer]>
	) {
		let lock = self.pool.lock().expect("mutex poisoned");

		log::trace!(
			"Freeing command buffers with {:#?} {:#?}",
			self,
			buffers.as_ref()
		);

		self.queue.device().free_command_buffers(
			*lock,
			buffers.as_ref()
		)
	}

	pub fn queue(&self) -> &Vrc<Queue> {
		&self.queue
	}

	pub fn device(&self) -> &Vrc<crate::device::Device> {
		self.queue.device()
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for CommandPool {
		type Target = Vutex<ash::vk::CommandPool> { pool }

		to_handle { .lock().expect("mutex poisoned").deref() }
	}
}
impl Drop for CommandPool {
	fn drop(&mut self) {
		let lock = self.pool.lock().expect("mutex poisoned");

		unsafe {
			self.queue.device().destroy_command_pool(*lock, self.allocation_callbacks.as_ref())
		}
	}
}
impl Debug for CommandPool {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct("CommandPool")
			.field("queue", &self.queue)
			.field("pool", &self.pool)
			.field("allocation_callbacks", &self.allocation_callbacks)
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