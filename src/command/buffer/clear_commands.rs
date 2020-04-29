use ash::vk;
use ash::version::DeviceV1_0;

use crate::prelude::{Image, ImageLayoutClearColorImage, Transparent, HasHandle};

use crate::resource::image::params::ImageSubresourceRangeTransparent;

impl<'a> super::recording::CommandBufferRecordingLock<'a> {
	pub fn clear_color_image(
		&self,
		image: &Image,
		layout: ImageLayoutClearColorImage,
		clear_color_value: &vk::ClearColorValue,
		ranges: &[ImageSubresourceRangeTransparent]
	) {
		unsafe {
			self.buffer.pool().device().cmd_clear_color_image(
				*self.lock,
				image.handle(),
				layout.into(),
				clear_color_value,
				Transparent::transmute_slice_twice(ranges)
			)
		}
	}
}