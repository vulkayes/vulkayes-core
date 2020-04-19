use std::{fmt, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::{
	device::Device,
	memory::host::HostMemoryAllocator,
	queue::Queue,
	util::sync::Vutex,
	Vrc
};

use super::error::{CommandBufferError, CommandPoolError};

macro_rules! impl_allocate_command_buffers_array {
	(
		$name: ident, $size: expr
	) => {
		/// Allocates command buffers into fixed-size array.
		///
		/// ### Safety
		///
		/// `level` must be a valid `vk::CommandBufferLevel` value.
		///
		/// ### Panic
		///
		/// This function will panic if the pool `Vutex` is poisoned.
		// Const generics can't come fast enough
		pub unsafe fn $name(
			&self,
			level: vk::CommandBufferLevel,
		) -> Result<[vk::CommandBuffer; $size], CommandBufferError> {
			let lock = self.pool.lock().expect("vutex poisoned");

			let alloc_info = vk::CommandBufferAllocateInfo::builder()
				.command_pool(*lock)
				.level(level)
				.command_buffer_count($size);

			log_trace_common!(
				"Allocating command buffers:",
				self,
				crate::util::fmt::format_handle(*lock),
				alloc_info.deref()
			);


			let mut buffers = std::mem::MaybeUninit::<[vk::CommandBuffer; $size]>::uninit();
			let err_code = self.device.fp_v1_0().allocate_command_buffers(
				self.device.handle(),
				alloc_info.deref() as *const _,
				buffers.as_mut_ptr() as *mut vk::CommandBuffer
			);
			match err_code {
				vk::Result::SUCCESS => Ok(buffers.assume_init()),
				_ => Err(CommandBufferError::from(err_code))
			}
		}
	};

	(
		$($name: ident, $size: expr),+
	) => {
		$(
			impl_allocate_command_buffers_array!($name, $size);
		)+
	}
}

/// Internally synchronized command pool.
pub struct CommandPool {
	device: Vrc<Device>,
	queue_family_index: u32,

	pool: Vutex<vk::CommandPool>,

	host_memory_allocator: HostMemoryAllocator
}
impl CommandPool {
	impl_allocate_command_buffers_array!(
		allocate_command_buffer,
		1,
		allocate_command_buffers_2,
		2,
		allocate_command_buffers_3,
		3,
		allocate_command_buffers_4,
		4,
		allocate_command_buffers_5,
		5,
		allocate_command_buffers_6,
		6,
		allocate_command_buffers_7,
		7,
		allocate_command_buffers_8,
		8
	);

	/// Note: `PROTECTED` flag value is currently ignored.
	pub fn new(
		queue: &Queue,
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
		let pool = queue
			.device()
			.create_command_pool(create_info.deref(), host_memory_allocator.as_ref())?;

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

		let flags = if return_resources {
			vk::CommandPoolResetFlags::RELEASE_RESOURCES
		} else {
			vk::CommandPoolResetFlags::empty()
		};

		unsafe {
			self.device
				.reset_command_pool(*lock, flags)
				.map_err(Into::into)
		}
	}

	/// Allocates multiple command buffers into a `Vec`.
	///
	/// ### Safety
	///
	/// `level` must be a valid `vk::CommandBufferLevel` value.
	///
	/// ### Panic
	///
	/// This function will panic if the pool `Vutex` is poisoned.
	pub unsafe fn allocate_command_buffers(
		&self,
		level: vk::CommandBufferLevel,
		count: std::num::NonZeroU32
	) -> Result<Vec<vk::CommandBuffer>, CommandBufferError> {
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

		self.device
			.allocate_command_buffers(alloc_info.deref())
			.map_err(Into::into)
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
	impl Deref, PartialEq, Eq, Hash for CommandPool {
		type Target = Vutex<vk::CommandPool> { pool }

		to_handle { .lock().expect("vutex poisoned").deref() }
	}
}
impl Drop for CommandPool {
	fn drop(&mut self) {
		let lock = self.pool.lock().expect("vutex poisoned");
		log_trace_common!("Dropping", self, lock);

		unsafe {
			self.device
				.destroy_command_pool(*lock, self.host_memory_allocator.as_ref())
		}
	}
}
impl fmt::Debug for CommandPool {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("CommandPool")
			.field("device", &self.device)
			.field("queue_family_index", &self.queue_family_index)
			.field("pool", &self.pool)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
