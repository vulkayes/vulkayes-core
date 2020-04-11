use std::ops::Deref;
use std::fmt;

use ash::vk;
use ash::version::DeviceV1_0;

use crate::{Vrc, device::Device};
use crate::memory::host::HostMemoryAllocator;

use super::error::DescriptorSetLayoutError;

pub mod params;

pub struct DescriptorSetLayout {
	device: Vrc<Device>,
	layout: vk::DescriptorSetLayout,
	host_memory_allocator: HostMemoryAllocator
}
impl DescriptorSetLayout {
	pub fn new(
		device: Vrc<Device>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, DescriptorSetLayoutError> {

		let create_info = vk::DescriptorSetLayoutCreateInfo::builder();
		unimplemented!();

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
		create_info: impl Deref<Target = vk::DescriptorSetLayoutCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, DescriptorSetLayoutError> {
		log_trace_common!(
			"Creating descriptor set layout:",
			device,
			create_info.deref(),
			host_memory_allocator
		);

		let layout = device.create_descriptor_set_layout(
			create_info.deref(),
			host_memory_allocator.as_ref()
		)?;

		Ok(
			Vrc::new(
				DescriptorSetLayout {
					device,
					layout,
					host_memory_allocator
				}
			)
		)
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for DescriptorSetLayout {
		type Target = vk::DescriptorSetLayout { layout }
	}
}
impl Drop for DescriptorSetLayout {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.device.destroy_descriptor_set_layout(
				self.layout,
				self.host_memory_allocator.as_ref()
			)
		}
	}
}
impl fmt::Debug for DescriptorSetLayout {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("DescriptorSetLayout")
			.field("device", &self.device)
			.field("layout", &crate::util::fmt::format_handle(self.layout))
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
