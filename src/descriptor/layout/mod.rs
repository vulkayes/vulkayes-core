use std::{fmt, ops::Deref};

use ash::vk;

use crate::prelude::{Device, HasHandle, HostMemoryAllocator, Vrc};

use super::error::DescriptorSetLayoutError;

pub mod params;

pub struct DescriptorSetLayout {
	device: Vrc<Device>,
	layout: vk::DescriptorSetLayout,

	host_memory_allocator: HostMemoryAllocator
}
impl DescriptorSetLayout {
	pub fn new<'a>(
		device: Vrc<Device>,
		flags: vk::DescriptorSetLayoutCreateFlags,
		bindings: impl Iterator<Item = params::DescriptorSetLayoutBinding<'a>>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, DescriptorSetLayoutError> {
		let bindings = collect_iter_faster!(
			bindings.enumerate().map(|(index, info)| {
				let builder: vk::DescriptorSetLayoutBindingBuilder = info.into();
				builder.binding(index as u32).build()
			}),
			8
		);

		let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.flags(flags)
			.bindings(bindings.as_slice());

		unsafe { Self::from_create_info(device, create_info, host_memory_allocator) }
	}

	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateDescriptorSetLayout.html>.
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

		let layout = device
			.create_descriptor_set_layout(create_info.deref(), host_memory_allocator.as_ref())?;

		Ok(Vrc::new(DescriptorSetLayout {
			device,
			layout,
			host_memory_allocator
		}))
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::DescriptorSetLayout>, Deref, Borrow, Eq, Hash, Ord for DescriptorSetLayout {
		target = { layout }
	}
}
impl Drop for DescriptorSetLayout {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.device
				.destroy_descriptor_set_layout(self.layout, self.host_memory_allocator.as_ref())
		}
	}
}
impl fmt::Debug for DescriptorSetLayout {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("DescriptorSetLayout")
			.field("device", &self.device)
			.field("layout", &self.safe_handle())
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
