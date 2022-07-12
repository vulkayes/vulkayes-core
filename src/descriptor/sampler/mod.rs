use std::{fmt, ops::Deref};

use ash::vk;

use crate::prelude::{Device, HasHandle, HostMemoryAllocator, Vrc};

pub mod params;

pub struct Sampler {
	device: Vrc<Device>,
	sampler: vk::Sampler,

	host_memory_allocator: HostMemoryAllocator
}
impl Sampler {
	pub fn new(
		device: Vrc<Device>,
		create_info: params::SamplerCreateInfo,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, super::error::SamplerError> {
		let create_info: vk::SamplerCreateInfoBuilder<'static> = create_info.into();

		unsafe {
			Self::from_create_info(
				device,
				create_info,
				host_memory_allocator
			)
		}
	}

	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::SamplerCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, super::error::SamplerError> {
		log_trace_common!(
			"Creating sampler:",
			device,
			create_info.deref(),
			host_memory_allocator
		);

		let sampler = device.create_sampler(
			create_info.deref(),
			host_memory_allocator.as_ref()
		)?;

		Ok(Vrc::new(Sampler {
			device,
			sampler,
			host_memory_allocator
		}))
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::Sampler>, Deref, Borrow, Eq, Hash, Ord for Sampler {
		target = { sampler }
	}
}
impl Drop for Sampler {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.device.destroy_sampler(
				self.sampler,
				self.host_memory_allocator.as_ref()
			)
		}
	}
}
impl fmt::Debug for Sampler {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Sampler")
			.field("device", &self.device)
			.field("sampler", &self.safe_handle())
			.field(
				"host_memory_allocator",
				&self.host_memory_allocator
			)
			.finish()
	}
}
