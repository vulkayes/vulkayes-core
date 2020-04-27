use std::{num::NonZeroU64, ops::DerefMut};

use ash::vk;

use crate::prelude::{Buffer, HasHandle, ImageView, SafeHandle, Sampler, Transparent};

use super::super::error::{
	DescriptorImageInfoError,
	DescriptorInlineUniformBlockInfoError,
	DescriptorSetWriteError
};

vk_builder_wrap! {
	/// Transparent wrapper struct over `DescriptorImageInfoBuilder`.
	pub struct DescriptorImageInfo ['a] {
		builder: vk::DescriptorImageInfoBuilder<'a> => vk::DescriptorImageInfo
	}
	impl ['a] {
		pub fn new(
			sampler: &'a Sampler,
			image_view: &'a ImageView,
			image_layout: vk::ImageLayout
		) -> Result<Self, DescriptorImageInfoError> {
			#[cfg(feature = "runtime_implicit_validations")]
			{
				if sampler.device() != image_view.image().device() {
					return Err(DescriptorImageInfoError::SamplerImageViewDeviceMismatch)
				}
			}

			Ok(Self {
				builder: vk::DescriptorImageInfo::builder()
					.sampler(sampler.handle())
					.image_view(image_view.handle())
					.image_layout(image_layout)
			})
		}

		pub fn with_immutable_sampler(
			image_view: &'a ImageView,
			image_layout: vk::ImageLayout
		) -> Self {
			Self {
				builder: vk::DescriptorImageInfo::builder()
					.image_view(image_view.handle())
					.image_layout(image_layout)
			}
		}
	}
}
vk_enum_subset! {
	pub enum DescriptorTypeImage {
		SAMPLER,
		COMBINED_IMAGE_SAMPLER,
		SAMPLED_IMAGE,
		STORAGE_IMAGE,
		INPUT_ATTACHMENT
	} impl Into<vk::DescriptorType>
}

vk_builder_wrap! {
	/// Transparent wrapper struct over `DescriptorBufferInfoBuilder`.
	pub struct DescriptorBufferInfo ['a] {
		builder: vk::DescriptorBufferInfoBuilder<'a> => vk::DescriptorBufferInfo
	}
	impl ['a] {
		pub fn new(buffer: &'a Buffer, offset: vk::DeviceSize, range: NonZeroU64) -> Self {
			DescriptorBufferInfo {
				builder: vk::DescriptorBufferInfo::builder()
					.buffer(buffer.handle())
					.offset(offset)
					.range(range.get())
			}
		}
	}
}
vk_enum_subset! {
	pub enum DescriptorTypeBuffer {
		UNIFORM_BUFFER,
		STORAGE_BUFFER,
		UNIFORM_BUFFER_DYNAMIC,
		STORAGE_BUFFER_DYNAMIC
	} impl Into<vk::DescriptorType>
}

vk_enum_subset! {
	pub enum DescriptorTypeTexelBuffer {
		UNIFORM_TEXEL_BUFFER,
		STORAGE_TEXEL_BUFFER
	} impl Into<vk::DescriptorType>
}

vk_builder_wrap! {
	/// Transparent wrapper struct over `WriteDescriptorSetInlineUniformBlockEXT`.
	pub struct DescriptorInlineUniformBlockInfo ['a] {
		builder: vk::WriteDescriptorSetInlineUniformBlockEXTBuilder<'a> => vk::WriteDescriptorSetInlineUniformBlockEXT
	}
	impl ['a] {
		pub fn new(data: &'a [u8]) -> Result<Self, DescriptorInlineUniformBlockInfoError> {
			#[cfg(feature = "runtime_implicit_validations")]
			{
				if data.len() == 0 {
					return Err(DescriptorInlineUniformBlockInfoError::DataEmpty)
				}

				if data.len() % 4 != 0 {
					return Err(DescriptorInlineUniformBlockInfoError::SizeNotMultipleOfFour)
				}
			}

			Ok(DescriptorInlineUniformBlockInfo {
				builder: vk::WriteDescriptorSetInlineUniformBlockEXT::builder().data(data)
			})
		}
	}
}
/// This is a hack. Waiting on `const_mut_refs` but it works like this on stable.
pub struct DescriptorInlineUniformBlockInfoRefMut<'a>(
	pub &'a mut DescriptorInlineUniformBlockInfo<'a>
);

unsafe_enum_variants! {
	enum DescriptorSetWriteDataInner ['a] {
		pub Image {
			descriptor_type: DescriptorTypeImage,
			image_infos: &'a [DescriptorImageInfo<'a>]
		} => {
			vk::WriteDescriptorSet::builder()
				.descriptor_type(descriptor_type.into())
				.image_info(
					Transparent::transmute_slice_twice(image_infos)
				)
		},

		pub Buffer {
			descriptor_type: DescriptorTypeBuffer,
			buffer_infos: &'a [DescriptorBufferInfo<'a>]
		} => {
			vk::WriteDescriptorSet::builder()
				.descriptor_type(descriptor_type.into())
				.buffer_info(
					Transparent::transmute_slice_twice(buffer_infos)
				)
		},

		pub TexelBuffer {
			descriptor_type: DescriptorTypeTexelBuffer,
			texel_buffer_views: &'a [SafeHandle<'a, vk::BufferView>]
		} => {
			vk::WriteDescriptorSet::builder()
				.descriptor_type(descriptor_type.into())
				.texel_buffer_view(
					Transparent::transmute_slice(texel_buffer_views)
				)
		},

		pub InlineUniformBlock {
			info: DescriptorInlineUniformBlockInfoRefMut<'a>
		} => {
			let mut builder = vk::WriteDescriptorSet::builder()
				.descriptor_type(vk::DescriptorType::INLINE_UNIFORM_BLOCK_EXT)
			;
			builder.descriptor_count = info.0.data_size;

			builder.push_next(
				info.0.deref_mut()
			)
		}
	} as pub DescriptorSetWriteData ['a] impl Into<vk::WriteDescriptorSetBuilder<'a>>
}

vk_builder_wrap! {
	/// Wrapper struct that is transparent `vk::WriteDescriptorSetBuilder`.
	pub struct DescriptorSetWrite ['a] {
		builder: vk::WriteDescriptorSetBuilder<'a> => vk::WriteDescriptorSet
	}
	impl ['a] {
		pub fn new(
			descriptor_set: SafeHandle<'a, vk::DescriptorSet>,
			binding: u32,
			array_element: u32,
			data: DescriptorSetWriteData<'a>
		) -> Result<Self, DescriptorSetWriteError> {
			let builder = Into::<vk::WriteDescriptorSetBuilder>::into(data)
				.dst_set(descriptor_set.into_handle())
				.dst_binding(binding)
				.dst_array_element(array_element);

			#[cfg(feature = "runtime_implicit_validations")]
			{
				if builder.descriptor_count == 0 {
					return Err(DescriptorSetWriteError::ZeroCount)
				}
			}

			Ok(DescriptorSetWrite { builder })
		}
	}
}

vk_builder_wrap! {
	/// Wrapper struct that is transparent `vk::WriteDescriptorSetBuilder`.
	pub struct DescriptorSetCopy ['a] {
		builder: vk::CopyDescriptorSetBuilder<'a> => vk::CopyDescriptorSet
	}
	impl ['a] {
		pub fn new(
			source_set: SafeHandle<'a, vk::DescriptorSet>,
			source_binding: u32,
			source_array_element: u32,
			destination_set: SafeHandle<'a, vk::DescriptorSet>,
			destination_binding: u32,
			destination_array_element: u32,
			count: u32
		) -> Self {
			DescriptorSetCopy {
				builder: vk::CopyDescriptorSet::builder()
					.src_set(source_set.into_handle())
					.src_binding(source_binding)
					.src_array_element(source_array_element)
					.dst_set(destination_set.into_handle())
					.dst_binding(destination_binding)
					.dst_array_element(destination_array_element)
					.descriptor_count(count)
			}
		}
	}
}
