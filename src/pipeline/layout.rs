use std::{fmt, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::prelude::{Device, HasHandle, HostMemoryAllocator, Vrc};

use super::error::PipelineLayoutError;
use crate::descriptor::layout::DescriptorSetLayout;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PushConstantRange {
	pub stage_flags: vk::ShaderStageFlags,
	pub offset_div_four: u32,
	pub size_div_four: std::num::NonZeroU32
}
impl Into<vk::PushConstantRange> for PushConstantRange {
	fn into(self) -> vk::PushConstantRange {
		vk::PushConstantRange::builder()
			.stage_flags(self.stage_flags)
			.offset(self.offset_div_four * 4)
			.size(self.size_div_four.get() * 4)
			.build()
	}
}

pub struct PipelineLayout {
	device: Vrc<Device>,
	layout: vk::PipelineLayout,

	descriptor_set_layouts: Vec<Vrc<DescriptorSetLayout>>,

	host_memory_allocator: HostMemoryAllocator
}
impl PipelineLayout {
	pub fn new(
		device: Vrc<Device>,
		descriptor_set_layouts: impl Iterator<Item = Vrc<DescriptorSetLayout>>,
		push_constant_ranges: impl Iterator<Item = PushConstantRange>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, PipelineLayoutError> {
		let descriptor_set_layouts: Vec<_> = collect_iter_faster!(descriptor_set_layouts, 8);

		let descriptor_set_layout_handles =
			collect_iter_faster!(descriptor_set_layouts.iter().map(|l| l.handle()), 8);
		let push_constant_ranges = collect_iter_faster!(
			push_constant_ranges.map(|r| Into::<vk::PushConstantRange>::into(r)),
			4
		);

		#[cfg(feature = "runtime_implicit_validations")]
		{
			for range in push_constant_ranges.iter() {
				if range.stage_flags == vk::ShaderStageFlags::empty() {
					return Err(PipelineLayoutError::StageFlagsEmpty)
				}
			}
		}

		let create_info = vk::PipelineLayoutCreateInfo::builder()
			.set_layouts(&descriptor_set_layout_handles)
			.push_constant_ranges(&push_constant_ranges);

		unsafe {
			Self::from_create_info(
				device,
				descriptor_set_layouts,
				create_info,
				host_memory_allocator
			)
		}
	}

	/// ### Safety
	///
	/// * See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreatePipelineLayout.html>.
	/// * `descriptor_set_layouts` must contain the same layouts as used in `create_info`.
	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		descriptor_set_layouts: Vec<Vrc<DescriptorSetLayout>>,
		create_info: impl Deref<Target = vk::PipelineLayoutCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, PipelineLayoutError> {
		log_trace_common!(
			"Creating pipeline layout:",
			device,
			create_info.deref(),
			host_memory_allocator
		);

		let layout =
			device.create_pipeline_layout(create_info.deref(), host_memory_allocator.as_ref())?;

		Ok(Vrc::new(PipelineLayout {
			device,
			layout,
			descriptor_set_layouts,
			host_memory_allocator
		}))
	}

	pub const fn descriptor_set_layouts(&self) -> &Vec<Vrc<DescriptorSetLayout>> {
		&self.descriptor_set_layouts
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::PipelineLayout>, Deref, Borrow, Eq, Hash, Ord for PipelineLayout {
		target = { layout }
	}
}
impl Drop for PipelineLayout {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.device
				.destroy_pipeline_layout(self.layout, self.host_memory_allocator.as_ref())
		}
	}
}
impl fmt::Debug for PipelineLayout {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("PipelineLayout")
			.field("device", &self.device)
			.field("layout", &self.safe_handle())
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
