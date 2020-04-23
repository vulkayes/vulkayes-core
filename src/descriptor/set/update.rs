use std::num::NonZeroU64;

use ash::vk;

use crate::prelude::{SafeHandle, ImageView, Sampler, HasHandle, Buffer};

use super::error::{DescriptorImageInfoError, DescriptorInlineUniformBlockInfoError};
use crate::util::transparent::Transparent;
use std::ops::{DerefMut, Deref};

/// Transparent wrapper struct over `DescriptorImageInfoBuilder` that guarantees validity of handles.
#[repr(transparent)]
pub struct DescriptorImageInfo<'a> {
	#[allow(dead_code)] // used through the Transparent trait
	builder: vk::DescriptorImageInfoBuilder<'a>
}
impl<'a> DescriptorImageInfo<'a> {
	pub const unsafe fn from_raw(builder: vk::DescriptorImageInfoBuilder<'a>) -> Self {
		DescriptorImageInfo {
			builder
		}
	}

	pub fn new(
		sampler: &'a Sampler,
		image_view: &'a ImageView,
		image_layout: vk::ImageLayout
	) -> Result<Self, DescriptorImageInfoError> {
		#[cfg(feature = "runtime_implicit_validations")]
		{
			if sampler.device() != image_view.image().device() {
				return Err(
					DescriptorImageInfoError::SamplerImageViewDeviceMismatch
				)
			}
		}

		Ok(
			Self {
				builder: vk::DescriptorImageInfo::builder()
					.sampler(sampler.handle())
					.image_view(image_view.handle())
					.image_layout(image_layout)
			}
		)
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
unsafe impl<'a> Transparent for DescriptorImageInfo<'a> {
	type Target = vk::DescriptorImageInfoBuilder<'a>;
}
unsafe impl<'a> Transparent for vk::DescriptorImageInfoBuilder<'a> {
	type Target = vk::DescriptorImageInfo;
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

/// Transparent wrapper struct over `DescriptorBufferInfoBuilder` that guarantees validity of handles.
#[repr(transparent)]
pub struct DescriptorBufferInfo<'a> {
	#[allow(dead_code)] // used through the Transparent trait
	builder: vk::DescriptorBufferInfoBuilder<'a>
}
impl<'a> DescriptorBufferInfo<'a> {
	pub const unsafe fn from_raw(builder: vk::DescriptorBufferInfoBuilder<'a>) -> Self {
		DescriptorBufferInfo {
			builder
		}
	}

	pub fn new(
		buffer: &'a Buffer,
		offset: vk::DeviceSize,
		range: NonZeroU64
	) -> Self {
		DescriptorBufferInfo {
			builder: vk::DescriptorBufferInfo::builder()
				.buffer(buffer.handle())
				.offset(offset)
				.range(range.get())
		}
	}

	pub fn transmute_slice(me: &[Self]) -> &[vk::DescriptorBufferInfoBuilder<'a>] {
		unsafe {
			std::mem::transmute(me)
		}
	}
}
unsafe impl<'a> Transparent for DescriptorBufferInfo<'a> {
	type Target = vk::DescriptorBufferInfoBuilder<'a>;
}
unsafe impl<'a> Transparent for vk::DescriptorBufferInfoBuilder<'a> {
	type Target = vk::DescriptorBufferInfo;
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

/// Transparent wrapper struct over `WriteDescriptorSetInlineUniformBlockEXT` that guarantees validity of handles.
#[repr(transparent)]
pub struct DescriptorInlineUniformBlockInfo<'a> {
	builder: vk::WriteDescriptorSetInlineUniformBlockEXTBuilder<'a>
}
impl<'a> DescriptorInlineUniformBlockInfo<'a> {
	pub const unsafe fn from_raw(builder: vk::WriteDescriptorSetInlineUniformBlockEXTBuilder<'a>) -> Self {
		DescriptorInlineUniformBlockInfo {
			builder
		}
	}

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

		Ok(
			DescriptorInlineUniformBlockInfo {
				builder: vk::WriteDescriptorSetInlineUniformBlockEXT::builder().data(data)
			}
		)
	}
}
impl<'a> Deref for DescriptorInlineUniformBlockInfo<'a> {
	type Target = vk::WriteDescriptorSetInlineUniformBlockEXTBuilder<'a>;

	fn deref(&self) -> &Self::Target {
		&self.builder
	}
}
impl<'a> DerefMut for DescriptorInlineUniformBlockInfo<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.builder
	}
}
unsafe impl<'a> Transparent for DescriptorInlineUniformBlockInfo<'a> {
	type Target = vk::WriteDescriptorSetInlineUniformBlockEXTBuilder<'a>;
}
unsafe impl<'a> Transparent for vk::WriteDescriptorSetInlineUniformBlockEXTBuilder<'a> {
	type Target = vk::WriteDescriptorSetInlineUniformBlockEXT;
}

/// This is a hack. Waiting on `const_mut_refs` but it works like this on stable.
pub struct DescriptorInlineUniformBlockInfoRefMut<'a>(pub &'a mut DescriptorInlineUniformBlockInfo<'a>);

unsafe_enum_variants! {
	enum DescriptorSetWriteDataInner ['a] {
		pub Image {
			descriptor_type: DescriptorTypeImage,
			image_infos: &'a [DescriptorImageInfo<'a>]
		} => {
			vk::WriteDescriptorSet::builder()
				.descriptor_type(descriptor_type.into())
				.image_info(
					Transparent::transmute_slice(
						Transparent::transmute_slice(image_infos)
					)
				)
		},

		pub Buffer {
			descriptor_type: DescriptorTypeBuffer,
			buffer_infos: &'a [DescriptorBufferInfo<'a>]
		} => {
			vk::WriteDescriptorSet::builder()
				.descriptor_type(descriptor_type.into())
				.buffer_info(
					Transparent::transmute_slice(
						Transparent::transmute_slice(buffer_infos)
					)
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
			vk::WriteDescriptorSet::builder()
				.descriptor_type(vk::DescriptorType::INLINE_UNIFORM_BLOCK_EXT)
				.push_next(
					info.0.deref_mut()
				)
		}
	} as pub DescriptorSetWriteData ['a] impl Into<vk::WriteDescriptorSetBuilder<'a>>
}

/// Wrapper struct that is transparent `vk::WriteDescriptorSetBuilder`, but contains validations.
#[repr(transparent)]
pub struct DescriptorSetWrite<'a> {
	#[allow(dead_code)] // Used through Transparent trait
	builder: vk::WriteDescriptorSetBuilder<'a>
}
impl<'a> DescriptorSetWrite<'a> {
	pub fn new(
		descriptor_set: SafeHandle<'a, vk::DescriptorSet>,
		binding: u32,
		array_element: u32,
		data: DescriptorSetWriteData<'a>
	) -> Self {
		let builder = Into::<vk::WriteDescriptorSetBuilder>::into(data)
			.dst_set(descriptor_set.into_handle())
			.dst_binding(binding)
			.dst_array_element(array_element)
		;

		DescriptorSetWrite {
			builder
		}
	}
}
unsafe impl<'a> Transparent for DescriptorSetWrite<'a> {
	type Target = vk::WriteDescriptorSetBuilder<'a>;
}
unsafe impl<'a> Transparent for vk::WriteDescriptorSetBuilder<'a> {
	type Target = vk::WriteDescriptorSet;
}

