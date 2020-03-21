use std::num::NonZeroU32;

use ash::vk;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ImageSize {
	image_type: vk::ImageType,
	width: NonZeroU32,
	height: NonZeroU32,
	depth: NonZeroU32,
	array_layers: NonZeroU32
}
impl ImageSize {
	pub const fn new_1d(width: NonZeroU32, array_layers: NonZeroU32) -> ImageSize1D {
		ImageSize1D(ImageSize {
			image_type: vk::ImageType::TYPE_1D,
			width,
			height: crate::NONZEROU32_ONE,
			depth: crate::NONZEROU32_ONE,
			array_layers
		})
	}

	pub const fn new_2d(
		width: NonZeroU32,
		height: NonZeroU32,
		array_layers: NonZeroU32
	) -> ImageSize2D {
		ImageSize2D(ImageSize {
			image_type: vk::ImageType::TYPE_2D,
			width,
			height,
			depth: crate::NONZEROU32_ONE,
			array_layers
		})
	}

	pub const fn new_3d(width: NonZeroU32, height: NonZeroU32, depth: NonZeroU32) -> ImageSize3D {
		ImageSize3D(ImageSize {
			image_type: vk::ImageType::TYPE_2D,
			width,
			height,
			depth,
			array_layers: crate::NONZEROU32_ONE
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

	pub fn complete_mipmap_chain_mipmaps(&self) -> NonZeroU32 {
		let max_dimension = std::cmp::max(std::cmp::max(self.width, self.height), self.depth);

		// SAFETY: log2(u32) + 1 cannot overflow u32 (at most 32 + 1), + 1 ensures non-zero
		unsafe {
			NonZeroU32::new_unchecked(((max_dimension.get() as f32).log2()).floor() as u32 + 1)
		}
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
	pub const fn new(size: NonZeroU32, layers_minus_6: u32) -> Self {
		ImageSizeCubeCompatible(ImageSize::new_2d(size, size, unsafe {
			NonZeroU32::new_unchecked(layers_minus_6 + 6)
		}))
	}
}
