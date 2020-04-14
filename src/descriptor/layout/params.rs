use std::num::NonZeroU32;

use ash::vk;

/// Enum for supported descriptor set layout types and don't require special handling.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(i32)]
pub enum DescriptorSetLayoutBindingGenericType {
	SAMPLED_IMAGE = vk::DescriptorType::SAMPLED_IMAGE.as_raw(),
	STORAGE_IMAGE = vk::DescriptorType::STORAGE_IMAGE.as_raw(),
	UNIFORM_TEXEL_BUFFER = vk::DescriptorType::UNIFORM_TEXEL_BUFFER.as_raw(),
	STORAGE_TEXEL_BUFFER = vk::DescriptorType::STORAGE_TEXEL_BUFFER.as_raw(),
	UNIFORM_BUFFER = vk::DescriptorType::UNIFORM_BUFFER.as_raw(),
	STORAGE_BUFFER = vk::DescriptorType::STORAGE_BUFFER.as_raw(),
	UNIFORM_BUFFER_DYNAMIC = vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC.as_raw(),
	STORAGE_BUFFER_DYNAMIC = vk::DescriptorType::STORAGE_BUFFER_DYNAMIC.as_raw()
}
impl Into<vk::DescriptorType> for DescriptorSetLayoutBindingGenericType {
	fn into(self) -> vk::DescriptorType {
		vk::DescriptorType::from_raw(self as i32)
	}
}

unsafe_enum_variants! {
	/// Statically typed validation requirements for different values of `vk::DescriptorType`.
	#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
	enum DescriptorSetLayoutBindingInner ['a] {
		/// Binding is reserved and must not be used from shaders.
		pub Reserved => { vk::DescriptorSetLayoutBinding::builder().descriptor_count(0) },

		/// Sampler or combined image sampler with the sampler part being immutable.
		pub ImmutableSamplers {
			combined: bool,
			stage_flags: vk::ShaderStageFlags,
			samplers: &'a [()] // TODO: Sampler object
		} => {
			vk::DescriptorSetLayoutBinding::builder()
				.descriptor_type(
					if combined { vk::DescriptorType::COMBINED_IMAGE_SAMPLER } else { vk::DescriptorType::SAMPLER }
				)
				.descriptor_count(samplers.len() as u32)
				.stage_flags(stage_flags)
				.immutable_samplers(unimplemented!())
		},

		/// Sampler or combined image sampler with mutable sampler.
		pub Samplers {
			combined: bool,
			count: NonZeroU32,
			stage_flags: vk::ShaderStageFlags
		} => {
			vk::DescriptorSetLayoutBinding::builder()
				.descriptor_type(
					if combined { vk::DescriptorType::COMBINED_IMAGE_SAMPLER } else { vk::DescriptorType::SAMPLER }
				)
				.descriptor_count(count.get())
				.stage_flags(stage_flags)
		},

		/// Inline uniform buffer, size is specified in bytes divided by four.
		pub InlineUniformBlock {
			size_div_four: NonZeroU32,
			stage_flags: vk::ShaderStageFlags
		} => {
			vk::DescriptorSetLayoutBinding::builder()
				.descriptor_type(vk::DescriptorType::INLINE_UNIFORM_BLOCK_EXT)
				.descriptor_count(size_div_four.get() * 4)
				.stage_flags(stage_flags)
		},

		/// Input attachment, limited to only FRAGMENT shader stage.
		pub InputAttachment {
			count: NonZeroU32
		} => {
			vk::DescriptorSetLayoutBinding::builder()
				.descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
				.descriptor_count(count.get())
				.stage_flags(vk::ShaderStageFlags::FRAGMENT)
		},

		pub Generic {
			descriptor_type: DescriptorSetLayoutBindingGenericType,
			count: NonZeroU32,
			stage_flags: vk::ShaderStageFlags
		} => {
			vk::DescriptorSetLayoutBinding::builder()
				.descriptor_type(descriptor_type.into())
				.descriptor_count(count.get())
				.stage_flags(stage_flags)
		},

		/// Custom combination.
		///
		/// ### Safety
		///
		/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkDescriptorSetLayoutBinding.html>.
		{unsafe} pub Custom {
			descriptor_type: vk::DescriptorType,
			count: NonZeroU32,
			stage_flags: vk::ShaderStageFlags
		} => {
			vk::DescriptorSetLayoutBinding::builder()
				.descriptor_type(descriptor_type)
				.descriptor_count(count.get())
				.stage_flags(stage_flags)
		}
	} as pub DescriptorSetLayoutBinding ['a] impl Into<vk::DescriptorSetLayoutBindingBuilder<'a>>
}
