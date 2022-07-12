use std::num::NonZeroU32;

use ash::vk;

use crate::prelude::{Buffer, HasHandle, Image, ImageLayoutDestination, Transparent};

vk_builder_wrap! {
	pub struct ImageSubresourceLayers {
		builder: vk::ImageSubresourceLayersBuilder<'static> => vk::ImageSubresourceLayers
	}
	impl {
		pub fn new(
			aspect_mask: vk::ImageAspectFlags,
			mip_level: u32,
			base_array_layer: u32,
			layer_count: NonZeroU32
		) -> Self {
			ImageSubresourceLayers {
				builder: vk::ImageSubresourceLayers::builder()
					.aspect_mask(aspect_mask)
					.mip_level(mip_level)
					.base_array_layer(base_array_layer)
					.layer_count(layer_count.get())
			}
		}
	}
}

vk_builder_wrap! {
	pub struct BufferImageCopy {
		builder: vk::BufferImageCopyBuilder<'static> => vk::BufferImageCopy
	}
	impl {
		pub fn new(
			buffer_offset: u64,
			buffer_dims: Option<[NonZeroU32; 2]>,
			image_subresource: ImageSubresourceLayers,
			image_offset: vk::Offset3D,
			image_extent: vk::Extent3D
		) -> Self {
			let mut builder = vk::BufferImageCopy::builder()
				.buffer_offset(buffer_offset)
				.image_subresource(
					image_subresource.transmute().transmute()
				)
				.image_offset(image_offset)
				.image_extent(image_extent)
			;

			if let Some([width, height]) = buffer_dims {
				builder = builder
					.buffer_row_length(width.get())
					.buffer_image_height(height.get())
			}

			BufferImageCopy {
				builder
			}
		}
	}
}

impl<'a> super::super::CommandBufferRecordingLockOutsideRenderPass<'a> {
	pub fn copy_buffer_to_image(
		&self,
		source: &Buffer,
		destination: &Image,
		destination_layout: ImageLayoutDestination,
		regions: impl AsRef<[BufferImageCopy]>
	) {
		log_trace_common!(
			"Copy buffer to image:",
			crate::util::fmt::format_handle(self.handle()),
			source,
			destination,
			destination_layout,
			regions.as_ref()
		);

		unsafe {
			self.device().cmd_copy_buffer_to_image(
				self.handle(),
				source.handle(),
				destination.handle(),
				destination_layout.into(),
				Transparent::transmute_slice_twice(regions.as_ref())
			)
		}
	}
}
