//! Swapchain is a set of image buffers which handles presentation and tearing.

use std::{
	num::NonZeroU32,
	ops::Deref,
	fmt::{Debug, self}
};
use std::mem::ManuallyDrop;

use crate::{
	ash::vk,
	device::Device,
	memory::host::HostMemoryAllocator,
	surface::Surface,
	Vrc
};
use crate::resource::image::Image;
use crate::resource::ImageSize;
use crate::queue::sharing_mode::SharingMode;

pub mod error;

#[derive(Debug)]
pub struct SwapchainImage {
	swapchain: Vrc<Swapchain>,
	// Image must not be dropped because it is managed by the Vulkan implementation.
	image: ManuallyDrop<Image>
}
impl SwapchainImage {
	/// Crates a new swapchain image.
	///
	/// ### Safety
	///
	/// `image` must be an image crated from `swapchain` using `.get_swapchain_images`.
	pub unsafe fn new(swapchain: Vrc<Swapchain>, image: Image) -> Self {
		SwapchainImage {
			swapchain,
			image: ManuallyDrop::new(image)
		}
	}

	pub fn swapchain(&self) -> &Vrc<Swapchain> {
		&self.swapchain
	}
}
impl Deref for SwapchainImage {
	type Target = Image;

	fn deref(&self) -> &Self::Target {
		&self.image
	}
}

/// Return type of `Swapchain` constructors.
#[derive(Debug)]
pub struct SwapchainData {
	pub swapchain: Vrc<Swapchain>,
	pub images: Vec<Vrc<SwapchainImage>>
}

#[derive(Debug)]
pub struct SwapchainCreateImageInfo {
	pub min_image_count: NonZeroU32,
	pub image_format: vk::Format,
	pub image_color_space: vk::ColorSpaceKHR,
	pub image_extent: [NonZeroU32; 2],
	pub image_array_layers: NonZeroU32,
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
			.image_extent(vk::Extent2D {
				width: self.image_extent[0].get(),
				height: self.image_extent[1].get()
			})
			.image_array_layers(self.image_array_layers.get())
			.image_usage(self.image_usage)
	}
}

pub struct Swapchain {
	surface: Vrc<Surface>,

	device: Vrc<Device>,
	loader: ash::extensions::khr::Swapchain,
	swapchain: vk::SwapchainKHR,

	allocation_callbacks: Option<vk::AllocationCallbacks>
}
impl Swapchain {
	pub fn new(
		device: Vrc<Device>,
		surface: Surface,
		image_info: SwapchainCreateImageInfo,
		sharing_mode: SharingMode<impl AsRef<[u32]>>,
		pre_transform: vk::SurfaceTransformFlagsKHR,
		composite_alpha: vk::CompositeAlphaFlagsKHR,
		present_mode: vk::PresentModeKHR,
		clipped: bool,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<SwapchainData, error::SwapchainError> {
		let create_info = vk::SwapchainCreateInfoKHR::builder()
			.surface(*surface)
			.pre_transform(pre_transform)
			.composite_alpha(composite_alpha)
			.present_mode(present_mode)
			.clipped(clipped)
			.image_sharing_mode(sharing_mode.sharing_mode())
			.queue_family_indices(sharing_mode.indices())
			;

		let create_info = image_info.add_to_create_info(create_info);

		unsafe {
			Self::from_create_info(
				device,
				Vrc::new(surface),
				create_info,
				host_memory_allocator
			)
		}
	}

	pub fn recreate(
		&self,
		image_info: SwapchainCreateImageInfo,
		sharing_mode: SharingMode<impl AsRef<[u32]>>,
		pre_transform: vk::SurfaceTransformFlagsKHR,
		composite_alpha: vk::CompositeAlphaFlagsKHR,
		present_mode: vk::PresentModeKHR,
		clipped: bool,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<SwapchainData, error::SwapchainError> {
		let create_info = vk::SwapchainCreateInfoKHR::builder()
			.surface(**self.surface)
			.pre_transform(pre_transform)
			.composite_alpha(composite_alpha)
			.present_mode(present_mode)
			.clipped(clipped)
			.old_swapchain(self.swapchain)
			.image_sharing_mode(sharing_mode.sharing_mode())
			.queue_family_indices(sharing_mode.indices())
			;

		let create_info = image_info.add_to_create_info(create_info);

		unsafe {
			Self::from_create_info(
				self.device.clone(),
				self.surface.clone(),
				create_info,
				host_memory_allocator
			)
		}
	}

	/// Creates a new `Swapchain` from an existing `SwapchainCreateInfoKHR`.
	///
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateSwapchainKHR.html>.
	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		surface: Vrc<Surface>,
		create_info: impl Deref<Target = vk::SwapchainCreateInfoKHR>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<SwapchainData, error::SwapchainError> {
		let loader = ash::extensions::khr::Swapchain::new(
			device.instance().deref().deref(),
			device.deref().deref()
		);

		let allocation_callbacks: Option<vk::AllocationCallbacks> = host_memory_allocator.into();

		let c_info = create_info.deref();
		let swapchain =
			loader.create_swapchain(c_info, allocation_callbacks.as_ref())?;

		let me = Vrc::new(Swapchain {
			surface,
			device: device.clone(),
			loader,
			swapchain,

			allocation_callbacks
		});

		let images: Vec<_> = me.loader.get_swapchain_images(swapchain)?
			.into_iter().map(|image| {
				Vrc::new(
					SwapchainImage::new(
						me.clone(),
						Image::from_existing(
							device.clone(),
							image,
							c_info.image_format,
							ImageSize::new_2d(
								NonZeroU32::new_unchecked(c_info.image_extent.width),
								NonZeroU32::new_unchecked(c_info.image_extent.height),
								NonZeroU32::new_unchecked(c_info.image_array_layers)
							)
						)
					)
				)
			})
			.collect();

		Ok(SwapchainData {
			swapchain: me,
			images
		})
	}

	pub fn device(&self) -> &Vrc<Device> {
		&self.device
	}

	pub fn surface(&self) -> &Vrc<Surface> {
		&self.surface
	}

	pub fn loader(&self) -> &ash::extensions::khr::Swapchain {
		&self.loader
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for Swapchain {
		type Target = vk::SwapchainKHR { swapchain }
	}
}
impl Drop for Swapchain {
	fn drop(&mut self) {
		unsafe {
			self.loader
				.destroy_swapchain(self.swapchain, self.allocation_callbacks.as_ref());
		}
	}
}
impl Debug for Swapchain {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Swapchain")
			.field("surface", &self.surface)
			.field("device", &self.device)
			.field("loader", &"<ash::extensions::khr::Swapchain>")
			.field(
				"swapchain",
				&crate::util::fmt::format_handle(self.swapchain)
			)
			.field("allocation_callbacks", &self.allocation_callbacks)
			.finish()
	}
}
