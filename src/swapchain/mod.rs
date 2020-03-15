//! Swapchain is a set of image buffers which handles presentation and tearing.

use std::{
	fmt::{self, Debug},
	mem::ManuallyDrop,
	num::NonZeroU32,
	ops::Deref
};

use crate::{
	ash::vk,
	device::Device,
	memory::host::HostMemoryAllocator,
	queue::{sharing_mode::SharingMode, Queue},
	resource::{image::Image, ImageSize},
	surface::Surface,
	util::sync::Vutex,
	Vrc
};

pub mod error;

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
	/// `image` must be an image crated from `swapchain` using `.get_swapchain_images`.
	/// `index` must be the index of the image as returned by the `.get_swapchain_images`.
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
	swapchain: Vutex<vk::SwapchainKHR>,
	retired: bool,

	host_memory_allocator: HostMemoryAllocator
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
			.queue_family_indices(sharing_mode.indices());

		if cfg!(feature = "runtime_implicit_validations") {
			if image_info.image_usage.is_empty() {
				return Err(error::SwapchainError::ImageUsageEmpty)
			}
		}
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
		let lock = self.swapchain.lock().expect("vutex poisoned");
		if self.retired {
			return Err(error::SwapchainError::SwapchainRetired)
		}

		let create_info = vk::SwapchainCreateInfoKHR::builder()
			.surface(**self.surface)
			.pre_transform(pre_transform)
			.composite_alpha(composite_alpha)
			.present_mode(present_mode)
			.clipped(clipped)
			.old_swapchain(*lock)
			.image_sharing_mode(sharing_mode.sharing_mode())
			.queue_family_indices(sharing_mode.indices());

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

		let c_info = create_info.deref();

		log_trace_common!(
			"Creating swapchain:",
			device,
			surface,
			c_info,
			host_memory_allocator
		);
		let swapchain = loader.create_swapchain(c_info, host_memory_allocator.as_ref())?;

		let me = Vrc::new(Swapchain {
			surface,
			device: device.clone(),
			loader,
			swapchain: Vutex::new(swapchain),
			retired: false,

			host_memory_allocator
		});

		let images: Vec<_> = me
			.loader
			.get_swapchain_images(swapchain)? // This is still okay since we haven't given anyone else access to the `swapchain` or `me` object, no synchronization problem
			.into_iter().enumerate()
			.map(|(index, image)| {
				Vrc::new(SwapchainImage::new(
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
					),
					index as u32
				))
			})
			.collect();

		Ok(SwapchainData {
			swapchain: me,
			images
		})
	}

	/// Presents on given queue.
	///
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkQueuePresentKHR.html>
	pub unsafe fn present(
		&self,
		queue: &Queue,
		info: impl Deref<Target = vk::PresentInfoKHR>
	) -> crate::queue::error::QueuePresentResult {
		let queue_lock = queue.lock().expect("queue Vutex poisoned");

		log_trace_common!("Presenting on queue:", self, queue_lock, info.deref());

		self.loader
			.queue_present(*queue_lock, info.deref())
			.map(Into::into)
			.map_err(Into::into)
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}

	pub const fn surface(&self) -> &Vrc<Surface> {
		&self.surface
	}

	pub const fn loader(&self) -> &ash::extensions::khr::Swapchain {
		&self.loader
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for Swapchain {
		type Target = Vutex<vk::SwapchainKHR> { swapchain }

		to_handle { .lock().expect("vutex poisoned").deref() }
	}
}
impl Drop for Swapchain {
	fn drop(&mut self) {
		let lock = self.swapchain.lock().expect("vutex poisoned");

		unsafe {
			self.loader
				.destroy_swapchain(*lock, self.host_memory_allocator.as_ref());
		}
	}
}
impl Debug for Swapchain {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Swapchain")
			.field("surface", &self.surface)
			.field("device", &self.device)
			.field("loader", &"<ash::extensions::khr::Swapchain>")
			.field("swapchain", &self.swapchain)
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
