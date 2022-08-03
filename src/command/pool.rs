use std::{fmt, ops::Deref, num::NonZeroU32};

use ash::vk;

use super::error::{CommandBufferError, CommandPoolError};
use crate::{device::Device, memory::host::HostMemoryAllocator, prelude::Vrc, queue::Queue, util::sync::Vutex};

/// Internally synchronized command pool.
pub struct CommandPool {
	device: Vrc<Device>,
	queue_family_index: u32,

	pool: Vutex<vk::CommandPool>,

	host_memory_allocator: HostMemoryAllocator
}
impl CommandPool {
	/// Note: `PROTECTED` flag value is currently ignored.
	pub fn new(queue: &Queue, flags: vk::CommandPoolCreateFlags, host_memory_allocator: HostMemoryAllocator) -> Result<Vrc<Self>, CommandPoolError> {
		let flags = flags & !vk::CommandPoolCreateFlags::PROTECTED;

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

	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateCommandPool.html>
	pub unsafe fn from_create_info(
		queue: &Queue,
		create_info: impl Deref<Target = vk::CommandPoolCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, CommandPoolError> {
		log_trace_common!(
			"Creating command pool:",
			queue,
			create_info.deref(),
			host_memory_allocator
		);
		let pool = queue.device().create_command_pool(
			create_info.deref(),
			host_memory_allocator.as_ref()
		)?;

		Ok(Vrc::new(Self {
			device: queue.device().clone(),
			queue_family_index: queue.queue_family_index(),

			pool: Vutex::new(pool),
			host_memory_allocator
		}))
	}

	/// ### Panic
	///
	/// * This function will panic if the pool `Vutex` is poisoned.
	/// * This function will panic under Vulkan 1.0.
	#[cfg(feature = "Vulkan1_1")]
	pub fn trim(&self) {
		use ash::version::DeviceV1_1;

		let lock = self.pool.lock().expect("vutex poisoned");

		unsafe {
			self.device
				.trim_command_pool(*lock, vk::CommandPoolTrimFlags::empty())
		}
	}

	/// ### Panic
	///
	/// This function will panic if the pool `Vutex` is poisoned.
	pub fn reset(&self, return_resources: bool) -> Result<(), CommandPoolError> {
		let lock = self.pool.lock().expect("vutex poisoned");

		let flags = if return_resources { vk::CommandPoolResetFlags::RELEASE_RESOURCES } else { vk::CommandPoolResetFlags::empty() };

		unsafe {
			self.device
				.reset_command_pool(*lock, flags)
				.map_err(Into::into)
		}
	}

	/// Allocates command buffers into fixed-size array.
	///
	/// ### Panic
	///
	/// This function will panic if the pool `Vutex` is poisoned.
	pub fn allocate_command_buffers<const BUFFERS: usize>(
		&self,
		secondary: bool,
	) -> Result<[vk::CommandBuffer; BUFFERS], CommandBufferError> {	
		let level  = if secondary {
			vk::CommandBufferLevel::SECONDARY
		} else {
			vk::CommandBufferLevel::PRIMARY
		};

		unsafe {
			let mut buffers = std::mem::MaybeUninit::<[vk::CommandBuffer; BUFFERS]>::uninit();
			self.allocate_command_buffers_into(
				level,
				NonZeroU32::new(BUFFERS as u32).unwrap(),
				buffers.as_mut_ptr() as *mut _
			)?;

			Ok(buffers.assume_init())
		}
	}

	/// Allocates multiple command buffers into existing memory.
	///
	/// ### Safety
	///
	/// * `level` must be a valid `vk::CommandBufferLevel` value.
	/// * `out` must point to memory with size for at least `count` elements.
	///
	/// ### Panic
	///
	/// This function will panic if the pool `Vutex` is poisoned.
	pub unsafe fn allocate_command_buffers_into(
		&self,
		level: vk::CommandBufferLevel,
		count: NonZeroU32,
		out: *mut vk::CommandBuffer,
	) -> Result<(), CommandBufferError> {
		let lock = self.pool.lock().expect("vutex poisoned");

		let alloc_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(*lock)
			.level(level)
			.command_buffer_count(count.get());

		log_trace_common!(
			"Allocating command buffers:",
			self,
			crate::util::fmt::format_handle(*lock),
			alloc_info.deref()
		);

		match self.device.fp_v1_0().allocate_command_buffers(
			self.device.handle(),
			alloc_info.deref() as *const _,
			out
		) {
			vk::Result::SUCCESS => Ok(()),
			err => Err(CommandBufferError::from(err))
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
			self,
			crate::util::fmt::format_handle(*lock),
			buffers.as_ref()
		);

		self.device.free_command_buffers(*lock, buffers.as_ref())
	}

	pub const fn queue_family_index(&self) -> u32 {
		self.queue_family_index
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasSynchronizedHandle<vk::CommandPool>, Borrow, Eq, Hash, Ord for CommandPool {
		target = { pool }
	}
}
impl Drop for CommandPool {
	fn drop(&mut self) {
		let lock = self.pool.lock().expect("vutex poisoned");
		log_trace_common!("Dropping", self, lock);

		unsafe {
			self.device.destroy_command_pool(
				*lock,
				self.host_memory_allocator.as_ref()
			)
		}
	}
}
impl fmt::Debug for CommandPool {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("CommandPool")
			.field("device", &self.device)
			.field(
				"queue_family_index",
				&self.queue_family_index
			)
			.field("pool", &self.pool)
			.field(
				"host_memory_allocator",
				&self.host_memory_allocator
			)
			.finish()
	}
}
