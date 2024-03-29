use std::{
	fmt::{self, Debug},
	ops::Deref
};

use ash::vk;

use crate::{device::Device, memory::host::HostMemoryAllocator, prelude::Vrc};

pub mod error;

/// A newtype for binary semaphores.
#[derive(Debug, Clone)]
pub struct BinarySemaphore(Vrc<Semaphore>);
impl BinarySemaphore {
	/// Creates a new binary semaphore wrapper.
	///
	/// ### Safety
	///
	/// `semaphore` must be a binary semaphore.
	pub unsafe fn new(semaphore: Vrc<Semaphore>) -> Self {
		BinarySemaphore(semaphore)
	}
}
impl Deref for BinarySemaphore {
	type Target = Vrc<Semaphore>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub struct Semaphore {
	device: Vrc<Device>,
	semaphore: vk::Semaphore,

	host_memory_allocator: HostMemoryAllocator
}
impl Semaphore {
	pub fn binary(device: Vrc<Device>, host_memory_allocator: HostMemoryAllocator) -> Result<BinarySemaphore, error::SemaphoreError> {
		let mut type_create_info = vk::SemaphoreTypeCreateInfo::builder().semaphore_type(vk::SemaphoreType::BINARY);

		let create_info = vk::SemaphoreCreateInfo::builder().push_next(&mut type_create_info);

		unsafe {
			Self::from_create_info(
				device,
				create_info,
				host_memory_allocator
			)
			.map_err(Into::into)
			.map(|s| BinarySemaphore::new(s))
		}
	}

	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateSemaphore.html>
	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::SemaphoreCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::SemaphoreError> {
		log_trace_common!(
			"Creating semaphore:",
			device,
			create_info.deref(),
			host_memory_allocator
		);
		let semaphore = device.create_semaphore(
			create_info.deref(),
			host_memory_allocator.as_ref()
		)?;

		Ok(Vrc::new(Semaphore {
			device,
			semaphore: semaphore,
			host_memory_allocator
		}))
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::Semaphore>, Deref, Borrow, Eq, Hash, Ord for Semaphore {
		target = { semaphore }
	}
}
impl Drop for Semaphore {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self, self.semaphore);

		unsafe {
			self.device.destroy_semaphore(
				self.semaphore,
				self.host_memory_allocator.as_ref()
			)
		}
	}
}
impl Debug for Semaphore {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Semaphore")
			.field("device", &self.device)
			.field("semaphore", &self.semaphore)
			.field(
				"host_memory_allocator",
				&self.host_memory_allocator
			)
			.finish()
	}
}
