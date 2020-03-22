use std::{mem::ManuallyDrop, num::NonZeroU32, ops::Deref};

use ash::vk;

use crate::{
	resource::image::{
		params::{ImageSize, ImageSize2D},
		Image
	},
	Vrc
};

use super::Swapchain;

#[derive(Debug, Copy, Clone)]
pub struct SwapchainCreateImageInfo {
	pub min_image_count: NonZeroU32,
	pub image_format: vk::Format,
	pub image_color_space: vk::ColorSpaceKHR,
	pub image_size: ImageSize2D,
	pub image_usage: vk::ImageUsageFlags
}
impl SwapchainCreateImageInfo {
	pub fn add_to_create_info<'a>(
		&'a self,
		builder: vk::SwapchainCreateInfoKHRBuilder<'a>
	) -> vk::SwapchainCreateInfoKHRBuilder<'a> {
		builder
			.min_image_count(self.min_image_count.get())
			.image_format(self.image_format)
			.image_color_space(self.image_color_space)
			.image_extent(ImageSize::from(self.image_size).into())
			.image_array_layers(ImageSize::from(self.image_size).array_layers().get())
			.image_usage(self.image_usage)
	}
}

#[derive(Debug)]
pub struct SwapchainImage {
	swapchain: Vrc<Swapchain>,
	// Image must not be dropped because it is managed by the Vulkan implementation.
	image: ManuallyDrop<Image>,
	/// Swapchain image index
	index: u32
}
impl SwapchainImage {
	/// Crates a new swapchain image.
	///
	/// ### Safety
	///
	/// * `image` must be an image crated from `swapchain` using `.get_swapchain_images`.
	/// * `index` must be the index of the image as returned by the `.get_swapchain_images`.
	pub unsafe fn new(swapchain: Vrc<Swapchain>, image: Image, index: u32) -> Self {
		SwapchainImage {
			swapchain,
			image: ManuallyDrop::new(image),
			index
		}
	}

	pub const fn swapchain(&self) -> &Vrc<Swapchain> {
		&self.swapchain
	}

	pub const fn index(&self) -> u32 {
		self.index
	}
}
impl Deref for SwapchainImage {
	type Target = Image;

	fn deref(&self) -> &Self::Target {
		&self.image
	}
}
