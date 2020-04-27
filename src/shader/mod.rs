use std::{fmt, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::prelude::{Device, HasHandle, HostMemoryAllocator, Vrc};

pub mod error;
pub mod params;

pub struct ShaderModule {
	device: Vrc<Device>,
	module: vk::ShaderModule,

	host_memory_allocator: HostMemoryAllocator
}
impl ShaderModule {
	pub fn new(
		device: Vrc<Device>,
		code: impl AsRef<[u32]>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::ShaderError> {
		let create_info = vk::ShaderModuleCreateInfo::builder().code(code.as_ref());

		unsafe { Self::from_create_info(device, create_info, host_memory_allocator) }
	}

	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::ShaderModuleCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, error::ShaderError> {
		log_trace_common!(
			"Creating shader module:",
			device,
			create_info.deref(),
			host_memory_allocator
		);

		let module =
			device.create_shader_module(create_info.deref(), host_memory_allocator.as_ref())?;

		Ok(Vrc::new(ShaderModule {
			device,
			module,

			host_memory_allocator
		}))
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::ShaderModule>, Deref, Borrow, Eq, Hash, Ord for ShaderModule {
		target = { module }
	}
}
impl Drop for ShaderModule {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.device
				.destroy_shader_module(self.module, self.host_memory_allocator.as_ref())
		}
	}
}
impl fmt::Debug for ShaderModule {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("ShaderModule")
			.field("device", &self.device)
			.field("module", &self.safe_handle())
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
