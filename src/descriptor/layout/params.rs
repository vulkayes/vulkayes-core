use std::num::NonZeroU32;

use ash::vk;

vk_enum_subset! {
	/// Enum for supported descriptor set layout types and don't require special handling.
	pub enum DescriptorSetLayoutBindingGenericType {
		SAMPLED_IMAGE,
		STORAGE_IMAGE,
		UNIFORM_TEXEL_BUFFER,
		STORAGE_TEXEL_BUFFER,
		UNIFORM_BUFFER,
		STORAGE_BUFFER,
		UNIFORM_BUFFER_DYNAMIC,
		STORAGE_BUFFER_DYNAMIC
	} impl Into<vk::DescriptorType>
}

unsafe_enum_variants! {
	/// Statically typed validation requirements for different values of `vk::DescriptorType`.
	#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
	enum DescriptorSetLayoutBindingInner ['a] {
		/// Binding is reserved and must not be used from shaders.
		pub Reserved => {
			vk::DescriptorSetLayoutBinding::builder().descriptor_count(0)
		},

		/// Sampler or combined image sampler with the sampler part being immutable.
		pub ImmutableSamplers {
			combined: bool,
			stage_flags: vk::ShaderStageFlags,
			samplers: &'a [crate::util::handle::SafeHandle<'a, vk::Sampler>] // TODO: Switch to owned to keep it alive?
		} => {
			vk::DescriptorSetLayoutBinding::builder()
				.descriptor_type(
					if combined { vk::DescriptorType::COMBINED_IMAGE_SAMPLER } else { vk::DescriptorType::SAMPLER }
				)
				.descriptor_count(samplers.len() as u32)
				.stage_flags(stage_flags)
				.immutable_samplers(
					unsafe { std::mem::transmute(samplers) }
				)
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
