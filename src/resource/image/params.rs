use std::num::NonZeroU32;

use ash::vk;

use super::size::{ImageSize, ImageSize2D, ImageSize3D, ImageSizeCubeCompatible};

unsafe_enum_variants! {
	enum MipmapLevelsInner {
		/// One mipmap level.
		pub One => { Some(crate::NONZEROU32_ONE) },
		/// Either `1` or `log2(max(width, height, depth)) + 1`, depending on `Image Creation Limits`.
		pub Most => { None },

		/// Custom number of mipmaps.
		///
		/// ### Safety
		///
		/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/html/vkspec.html#resources-image-creation-limits>.
		{unsafe} pub Custom { mipmaps: NonZeroU32 } => { Some(mipmaps) }
	} as pub MipmapLevels impl Into<Option<NonZeroU32>>
}

unsafe_enum_variants! {
	/// Statically typed common safe combinations of image size, mipmap levels, sample flags and image create flags.
	enum ImageSizeInfoInner {
		/// General image size with no flags enabled.
		pub General { size: ImageSize, mipmaps: MipmapLevels } => { (size, mipmaps, vk::SampleCountFlags::TYPE_1, vk::ImageCreateFlags::empty()) },

		/// Cube compatible 2D layered image.
		pub CubeCompatible { size: ImageSizeCubeCompatible, mipmaps: MipmapLevels } => { ( ImageSize2D::from(size).into(), mipmaps, vk::SampleCountFlags::TYPE_1, vk::ImageCreateFlags::empty()) },
		/// Multisampled 2D image.
		pub Multisampled { size: ImageSize2D, samples: vk::SampleCountFlags } => { (size.into(), MipmapLevels::One(), samples, vk::ImageCreateFlags::empty()) },
		/// Array compatible 3D image.
		pub ArrayCompatible { size: ImageSize3D } => { (size.into(), MipmapLevels::One(), vk::SampleCountFlags::TYPE_1, vk::ImageCreateFlags::empty()) },

		/// Custom combination.
		///
		/// ### Safety
		///
		/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkImageCreateInfo.html>
		{unsafe} pub Custom { size: ImageSize, mipmaps: MipmapLevels, samples: vk::SampleCountFlags, flags: vk::ImageCreateFlags } => { (size, mipmaps, samples, flags) }
	} as pub ImageSizeInfo impl Into<(ImageSize, MipmapLevels, vk::SampleCountFlags, vk::ImageCreateFlags)>
}

unsafe_enum_variants! {
	/// Statically typed common safe combinations of image tiling and layout.
	enum ImageTilingAndLayoutInner {
		/// Optimal tiling and undefined layout
		pub OptimalUndefined => { (vk::ImageTiling::OPTIMAL, vk::ImageLayout::UNDEFINED) },
		/// Linear tiling and preinitialized layout
		pub LinearPreinitialized => { (vk::ImageTiling::LINEAR, vk::ImageLayout::PREINITIALIZED) },

		/// Custom combination.
		///
		/// ### Safety
		///
		/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkImageCreateInfo.html>
		{unsafe} pub Custom { tiling: vk::ImageTiling, layout: vk::ImageLayout } => { (tiling, layout) }
	} as pub ImageTilingAndLayout impl Into<(vk::ImageTiling, vk::ImageLayout)>
}
