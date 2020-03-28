use std::num::NonZeroU32;

use ash::vk;

unsafe_enum_variants! {
	#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ImageSize {
	image_type: vk::ImageType,
	width: NonZeroU32,
	height: NonZeroU32,
	depth: NonZeroU32,
	array_layers: NonZeroU32,
	mipmap_levels: NonZeroU32
}
impl ImageSize {
	pub const unsafe fn new(
		image_type: vk::ImageType,
		width: NonZeroU32,
		height: NonZeroU32,
		depth: NonZeroU32,
		array_layers: NonZeroU32,
		mipmap_levels: NonZeroU32
	) -> Self {
		ImageSize {
			image_type,
			width,
			height,
			depth,
			array_layers,
			mipmap_levels
		}
	}

	pub fn new_1d(
		width: NonZeroU32,
		array_layers: NonZeroU32,
		mipmaps: MipmapLevels
	) -> ImageSize1D {
		let height = crate::NONZEROU32_ONE;
		let depth = crate::NONZEROU32_ONE;

		let mipmap_levels: Option<NonZeroU32> = mipmaps.into();
		let mipmap_levels = mipmap_levels
			.unwrap_or_else(|| Self::complete_mipmap_chain_mipmaps(width, height, depth));

		ImageSize1D(ImageSize {
			image_type: vk::ImageType::TYPE_1D,
			width,
			height,
			depth,
			array_layers,
			mipmap_levels
		})
	}

	pub fn new_2d(
		width: NonZeroU32,
		height: NonZeroU32,
		array_layers: NonZeroU32,
		mipmaps: MipmapLevels
	) -> ImageSize2D {
		let depth = crate::NONZEROU32_ONE;

		let mipmap_levels: Option<NonZeroU32> = mipmaps.into();
		let mipmap_levels = mipmap_levels
			.unwrap_or_else(|| Self::complete_mipmap_chain_mipmaps(width, height, depth));

		ImageSize2D(ImageSize {
			image_type: vk::ImageType::TYPE_2D,
			width,
			height,
			depth,
			array_layers,
			mipmap_levels
		})
	}

	pub fn new_3d(
		width: NonZeroU32,
		height: NonZeroU32,
		depth: NonZeroU32,
		mipmaps: MipmapLevels
	) -> ImageSize3D {
		let mipmap_levels: Option<NonZeroU32> = mipmaps.into();
		let mipmap_levels = mipmap_levels
			.unwrap_or_else(|| Self::complete_mipmap_chain_mipmaps(width, height, depth));

		ImageSize3D(ImageSize {
			image_type: vk::ImageType::TYPE_2D,
			width,
			height,
			depth,
			array_layers: crate::NONZEROU32_ONE,
			mipmap_levels
		})
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

	pub const fn mipmap_levels(&self) -> NonZeroU32 {
		self.mipmap_levels
	}

	pub fn complete_mipmap_chain_mipmaps(
		width: NonZeroU32,
		height: NonZeroU32,
		depth: NonZeroU32
	) -> NonZeroU32 {
		let max_dimension = std::cmp::max(std::cmp::max(width, height), depth);

		// SAFETY: log2(u32) + 1 cannot overflow u32 (at most 32 + 1), + 1 ensures non-zero
		unsafe {
			NonZeroU32::new_unchecked(((max_dimension.get() as f32).log2()).floor() as u32 + 1)
		}
	}

	/// ### Safety
	///
	/// * `info.extent.width` must be non-zero
	/// * `info.extent.height` must be non-zero
	/// * `info.extent.depth` must be non-zero
	/// * `info.array_layers` must be non-zero
	/// * `info.mip_levels` must be non-zero
	pub unsafe fn from_image_create_info(info: &vk::ImageCreateInfo) -> Self {
		let width = NonZeroU32::new_unchecked(info.extent.width);
		let height = NonZeroU32::new_unchecked(info.extent.height);
		let depth = NonZeroU32::new_unchecked(info.extent.depth);
		let array_layers = NonZeroU32::new_unchecked(info.array_layers);
		let mipmap_levels = NonZeroU32::new_unchecked(info.mip_levels);

		Self::new(
			info.image_type,
			width,
			height,
			depth,
			array_layers,
			mipmap_levels
		)
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
impl From<ImageSize1D> for ImageSize {
	fn from(value: ImageSize1D) -> Self {
		value.0
	}
}
impl From<ImageSize2D> for ImageSize {
	fn from(value: ImageSize2D) -> Self {
		value.0
	}
}
impl From<ImageSize3D> for ImageSize {
	fn from(value: ImageSize3D) -> Self {
		value.0
	}
}

/// Transparent image size wrapper that is guaranteed to be 1D.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ImageSize1D(ImageSize);
/// Transparent image size wrapper that is guaranteed to be 2D.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ImageSize2D(ImageSize);
impl From<ImageSizeCubeCompatible> for ImageSize2D {
	fn from(value: ImageSizeCubeCompatible) -> Self {
		value.0
	}
}
/// Transparent image size wrapper that is guaranteed to be 3D.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ImageSize3D(ImageSize);
/// Wrapper around `ImageSize` that is also guaranteed to be cube-compatible.
/// Cube compatible images must be 2D, must have square dimensions and must have at least 6 layers.
///
/// This wrapper guarantees that the image size upholds the invariants imposed
/// in Valid Usage section of <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkImageCreateInfo.html>
/// concerning `CUBE_COMPATIBLE_BIT`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ImageSizeCubeCompatible(ImageSize2D);
impl ImageSizeCubeCompatible {
	/// Creates a new `CubeCompatibleImageSize`.
	///
	/// `layers_minus_6` is the number of image layers minus six.
	/// Six is then added to the number of array layers in the constructor.
	///
	/// ### Panic
	///
	/// This method will panic if `layers_minus_6 + 6` overflows `u32`.
	pub fn new(size: NonZeroU32, layers_minus_6: u32, mipmaps: MipmapLevels) -> Self {
		ImageSizeCubeCompatible(ImageSize::new_2d(
			size,
			size,
			unsafe { NonZeroU32::new_unchecked(layers_minus_6 + 6) },
			mipmaps
		))
	}
}

unsafe_enum_variants! {
	/// Statically typed common safe combinations of image size, mipmap levels, sample flags and image create flags.
	#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
	enum ImageSizeInfoInner {
		/// General image size with no flags enabled.
		pub General { size: ImageSize } => { (size, vk::SampleCountFlags::TYPE_1, vk::ImageCreateFlags::empty()) },

		/// Cube compatible 2D layered image.
		pub CubeCompatible { size: ImageSizeCubeCompatible } => { ( ImageSize2D::from(size).into(), vk::SampleCountFlags::TYPE_1, vk::ImageCreateFlags::empty()) },
		/// Multisampled 2D image.
		pub Multisampled { width: NonZeroU32, height: NonZeroU32, array_layers: NonZeroU32, samples: vk::SampleCountFlags } => {
			(
				ImageSize::new_2d(
					width,
					height,
					array_layers,
					MipmapLevels::One()
				).into(),
				samples,
				vk::ImageCreateFlags::empty()
			)
		},
		/// Array compatible 3D image.
		pub ArrayCompatible { size: ImageSize3D } => {
			(
				size.into(),
				vk::SampleCountFlags::TYPE_1,
				vk::ImageCreateFlags::empty()
			)
		},

		/// Custom combination.
		///
		/// ### Safety
		///
		/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkImageCreateInfo.html>
		{unsafe} pub Custom { size: ImageSize, samples: vk::SampleCountFlags, flags: vk::ImageCreateFlags } => { (size, samples, flags) }
	} as pub ImageSizeInfo impl Into<(ImageSize, vk::SampleCountFlags, vk::ImageCreateFlags)>
}
impl From<ImageSize> for ImageSizeInfo {
	fn from(value: ImageSize) -> Self {
		ImageSizeInfo::General(value)
	}
}

unsafe_enum_variants! {
	/// Statically typed common safe combinations of image tiling and layout.
	#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
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
impl Default for ImageTilingAndLayout {
	fn default() -> Self {
		ImageTilingAndLayout::OptimalUndefined()
	}
}

unsafe_enum_variants! {
	#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
	enum ImageViewRangeInner {
		pub Type1D { mipmap_levels_base: u32, mipmap_levels: NonZeroU32, array_layers_base: u32 } => {
			ImageSubresourceSlice {
				view_type: vk::ImageViewType::TYPE_1D,
				mipmap_levels_base,
				mipmap_levels,
				array_layers_base,
				array_layers: crate::NONZEROU32_ONE
			}
		},
		pub Type1DArray { mipmap_levels_base: u32, mipmap_levels: NonZeroU32, array_layers_base: u32, array_layers: NonZeroU32 } => {
			ImageSubresourceSlice {
				view_type: vk::ImageViewType::TYPE_1D_ARRAY,
				mipmap_levels_base,
				mipmap_levels,
				array_layers_base,
				array_layers
			}
		},

		pub Type2D { mipmap_levels_base: u32, mipmap_levels: NonZeroU32, array_layers_base: u32 } => {
			ImageSubresourceSlice {
				view_type: vk::ImageViewType::TYPE_2D,
				mipmap_levels_base,
				mipmap_levels,
				array_layers_base,
				array_layers: crate::NONZEROU32_ONE
			}
		},
		pub Type2DArray { mipmap_levels_base: u32, mipmap_levels: NonZeroU32, array_layers_base: u32, array_layers: NonZeroU32 } => {
			ImageSubresourceSlice {
				view_type: vk::ImageViewType::TYPE_2D_ARRAY,
				mipmap_levels_base,
				mipmap_levels,
				array_layers_base,
				array_layers
			}
		},

		pub TypeCube { mipmap_levels_base: u32, mipmap_levels: NonZeroU32, array_layers_base: u32 } => {
			ImageSubresourceSlice {
				view_type: vk::ImageViewType::CUBE,
				mipmap_levels_base,
				mipmap_levels,
				array_layers_base,
				array_layers: unsafe { NonZeroU32::new_unchecked(6) }
			}
		},
		/// The value of `array_layers` will be calculated as `array_layers_mult * 6`.
		pub TypeCubeArray { mipmap_levels_base: u32, mipmap_levels: NonZeroU32, array_layers_base: u32, array_layers_mult: NonZeroU32 } => {
			ImageSubresourceSlice {
				view_type: vk::ImageViewType::CUBE_ARRAY,
				mipmap_levels_base,
				mipmap_levels,
				array_layers_base,
				array_layers: unsafe { NonZeroU32::new_unchecked(array_layers_mult.get() * 6) }
			}
		},

		pub Type3D { mipmap_levels_base: u32, mipmap_levels: NonZeroU32 } => {
			ImageSubresourceSlice {
				view_type: vk::ImageViewType::TYPE_3D,
				mipmap_levels_base,
				mipmap_levels,
				array_layers_base: 0,
				array_layers: crate::NONZEROU32_ONE
			}
		},

		{unsafe} pub Custom { view_type: vk::ImageViewType, mipmap_levels_base: u32, mipmap_levels: NonZeroU32, array_layers_base: u32, array_layers: NonZeroU32 } => {
			ImageSubresourceSlice {
				view_type,
				mipmap_levels_base,
				mipmap_levels,
				array_layers_base,
				array_layers
			}
		}
	} as pub ImageViewRange impl Into<ImageSubresourceSlice>
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ImageSubresourceSlice {
	pub view_type: vk::ImageViewType,
	pub mipmap_levels_base: u32,
	pub mipmap_levels: NonZeroU32,
	pub array_layers_base: u32,
	pub array_layers: NonZeroU32
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ImageSubresourceRange {
	pub aspect_mask: vk::ImageAspectFlags,
	pub mipmap_levels_base: u32,
	pub mipmap_levels: NonZeroU32,
	pub array_layers_base: u32,
	pub array_layers: NonZeroU32
}
impl ImageSubresourceRange {
	/// ### Safety
	///
	/// * `info.subresource_range.level_count` must be non-zero
	/// * `info.subresource_range.layer_count` must be non-zero
	pub const unsafe fn from_image_view_create_info(info: &vk::ImageViewCreateInfo) -> Self {
		ImageSubresourceRange {
			aspect_mask: info.subresource_range.aspect_mask,
			mipmap_levels_base: info.subresource_range.base_mip_level,
			mipmap_levels: NonZeroU32::new_unchecked(info.subresource_range.level_count),
			array_layers_base: info.subresource_range.base_array_layer,
			array_layers: NonZeroU32::new_unchecked(info.subresource_range.layer_count)
		}
	}
}
