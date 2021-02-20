use std::{fmt, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::prelude::{Device, HasHandle, HostMemoryAllocator, SafeHandle, Transparent, Vrc};

use super::error::PipelineLayoutError;

vk_builder_wrap! {
	pub struct PushConstantRange {
		builder: vk::PushConstantRangeBuilder<'static> => vk::PushConstantRange
	}
	impl {
		pub fn new(
			stage_flags: vk::ShaderStageFlags,
			offset_div_four: u32,
			size_div_four: std::num::NonZeroU32
		) -> Self {
			let builder = vk::PushConstantRange::builder()
				.stage_flags(stage_flags)
				.offset(offset_div_four * 4)
				.size(size_div_four.get() * 4)
			;

			PushConstantRange {
				builder
			}
		}
	}
}

pub struct PipelineLayout {
	device: Vrc<Device>,
	layout: vk::PipelineLayout,

	host_memory_allocator: HostMemoryAllocator
}
impl PipelineLayout {
	pub fn new<'a>(
		device: Vrc<Device>,
		descriptor_set_layouts: impl AsRef<[SafeHandle<'a, vk::DescriptorSetLayout>]>,
		push_constant_ranges: impl AsRef<[PushConstantRange]>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, PipelineLayoutError> {
		#[cfg(feature = "runtime_implicit_validations")]
		{
			for range in push_constant_ranges.as_ref().iter() {
				if range.stage_flags == vk::ShaderStageFlags::empty() {
					return Err(PipelineLayoutError::StageFlagsEmpty)
				}
			}
		}

		let create_info = vk::PipelineLayoutCreateInfo::builder()
			.set_layouts(
				Transparent::transmute_slice(
					descriptor_set_layouts.as_ref()
				)
			)
			.push_constant_ranges(
				Transparent::transmute_slice_twice(
					push_constant_ranges.as_ref()
				)
			);

		unsafe {
			Self::from_create_info(
				device,
				create_info,
				host_memory_allocator
			)
		}
	}

	/// ### Safety
	///
	/// * See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreatePipelineLayout.html>.
	pub unsafe fn from_create_info(
		device: Vrc<Device>,
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
			host_memory_allocator
		}))
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
