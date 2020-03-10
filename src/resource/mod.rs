//! Resources are both buffers and images

use std::num::NonZeroU32;

use ash::vk;

pub mod image;

unsafe_enum_variants! {
	enum MipmapLevelsInner {
		/// One mipmap level.
		pub One,
		/// Either one or `log2(max(width, height, depth)) + 1`, depending on `Image Creation Limits`.
		pub Most,
		/// Custom number of mipmaps.
		///
		/// ### Safety
		///
		/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/html/vkspec.html#resources-image-creation-limits>.
		{unsafe} pub Custom(NonZeroU32)
	} as pub MipmapLevels
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ImageSize {
	image_type: vk::ImageType,
	width: NonZeroU32,
	height: NonZeroU32,
	depth: NonZeroU32,
	array_layers: NonZeroU32
}
impl ImageSize {
	pub const fn new_1d(width: NonZeroU32, array_layers: NonZeroU32) -> Self {
		ImageSize {
			image_type: vk::ImageType::TYPE_1D,
			width,
			height: crate::NONZEROU32_ONE,
			depth: crate::NONZEROU32_ONE,
			array_layers
		}
	}

	pub const fn new_2d(width: NonZeroU32, height: NonZeroU32, array_layers: NonZeroU32) -> Self {
		ImageSize {
			image_type: vk::ImageType::TYPE_2D,
			width,
			height,
			depth: crate::NONZEROU32_ONE,
			array_layers
		}
	}

	pub const fn new_3d(width: NonZeroU32, height: NonZeroU32, depth: NonZeroU32) -> Self {
		ImageSize {
			image_type: vk::ImageType::TYPE_2D,
			width,
			height,
			depth,
			array_layers: crate::NONZEROU32_ONE
		}
	}

	pub const fn image_type(&self) -> vk::ImageType {
		self.image_type
	}

	pub const fn extent(&self) -> [NonZeroU32; 3] {
		[self.width, self.height, self.depth]
	}

	pub const fn width(&self) -> NonZeroU32 {
		self.width
	}

	pub const fn height(&self) -> NonZeroU32 {
		self.height
	}

	pub const fn depth(&self) -> NonZeroU32 {
		self.depth
	}

	pub const fn array_layers(&self) -> NonZeroU32 {
		self.array_layers
	}
}
impl Into<vk::Extent3D> for ImageSize {
	fn into(self) -> vk::Extent3D {
		vk::Extent3D {
			width: self.width.get(),
			height: self.height.get(),
			depth: self.depth.get()
		}
	}
}
impl Into<vk::Extent2D> for ImageSize {
	fn into(self) -> vk::Extent2D {
		vk::Extent2D {
			width: self.width.get(),
			height: self.height.get()
		}
	}
}
