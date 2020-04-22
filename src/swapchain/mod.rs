//! Swapchain is a set of image buffers which handles presentation and tearing.

use std::{
	fmt::{self, Debug},
	num::NonZeroU32,
	ops::Deref
};

use crate::{
	ash::vk,
	device::Device,
	memory::host::HostMemoryAllocator,
	queue::{sharing_mode::SharingMode, Queue},
	resource::image::{
		params::{ImageSize, MipmapLevels},
		Image
	},
	surface::Surface,
	sync::{fence::Fence, semaphore::BinarySemaphore},
	util::sync::{AtomicVool, Vutex},
	prelude::Vrc
};

pub mod error;
pub mod image;

#[derive(Debug)]
pub enum AcquireSynchronization<'a> {
	Semaphore(&'a BinarySemaphore),
	Fence(&'a Fence),
	Both(&'a BinarySemaphore, &'a Fence)
}
impl<'a> AcquireSynchronization<'a> {
	pub fn fence(&self) -> Option<&Fence> {
		match self {
			AcquireSynchronization::Semaphore(_) => None,
			AcquireSynchronization::Fence(f) => Some(f),
			AcquireSynchronization::Both(_, f) => Some(f)
		}
	}

	pub fn semaphore(&self) -> Option<&BinarySemaphore> {
		match self {
			AcquireSynchronization::Semaphore(s) => Some(s),
			AcquireSynchronization::Fence(_) => None,
			AcquireSynchronization::Both(s, _) => Some(s)
		}
	}
}
impl<'a> From<&'a Fence> for AcquireSynchronization<'a> {
	fn from(value: &'a Fence) -> Self {
		AcquireSynchronization::Fence(value)
	}
}
impl<'a> From<&'a BinarySemaphore> for AcquireSynchronization<'a> {
	fn from(value: &'a BinarySemaphore) -> Self {
		AcquireSynchronization::Semaphore(value)
	}
}
impl<'a> From<(&'a BinarySemaphore, &'a Fence)> for AcquireSynchronization<'a> {
	fn from(value: (&'a BinarySemaphore, &'a Fence)) -> Self {
		AcquireSynchronization::Both(value.0, value.1)
	}
}

/// Return type of `Swapchain` constructors.
#[derive(Debug)]
pub struct SwapchainData {
	pub swapchain: Vrc<Swapchain>,
	pub images: Vec<Vrc<image::SwapchainImage>>
}

#[derive(Debug, Copy, Clone)]
pub struct SwapchainCreateInfo<A: AsRef<[u32]>> {
	pub image_info: image::SwapchainCreateImageInfo,
	pub sharing_mode: SharingMode<A>,
	pub pre_transform: vk::SurfaceTransformFlagsKHR,
	pub composite_alpha: vk::CompositeAlphaFlagsKHR,
	pub present_mode: vk::PresentModeKHR,
	pub clipped: bool
}

pub struct Swapchain {
	surface: Vrc<Surface>,

	device: Vrc<Device>,
	loader: ash::extensions::khr::Swapchain,
	swapchain: Vutex<vk::SwapchainKHR>,
	retired: AtomicVool,

	host_memory_allocator: HostMemoryAllocator
}
impl Swapchain {
	pub fn new(
		device: Vrc<Device>,
		surface: Surface,
		create_info: SwapchainCreateInfo<impl AsRef<[u32]>>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<SwapchainData, error::SwapchainError> {
		#[cfg(feature = "runtime_implicit_validations")]
		{
			if create_info.image_info.image_usage.is_empty() {
				return Err(error::SwapchainError::ImageUsageEmpty)
			}
		}

		let c_info = vk::SwapchainCreateInfoKHR::builder()
			.surface(*surface)
			.pre_transform(create_info.pre_transform)
			.composite_alpha(create_info.composite_alpha)
			.present_mode(create_info.present_mode)
			.clipped(create_info.clipped)
			.image_sharing_mode(create_info.sharing_mode.sharing_mode())
			.queue_family_indices(create_info.sharing_mode.indices());

		let c_info = create_info.image_info.add_to_create_info(c_info);

		unsafe { Self::from_create_info(device, Vrc::new(surface), c_info, host_memory_allocator) }
	}

	pub fn recreate(
		&self,
		create_info: SwapchainCreateInfo<impl AsRef<[u32]>>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<SwapchainData, error::SwapchainError> {
		let lock = self.swapchain.lock().expect("vutex poisoned");
		// Safe because of the vutex above
		if self.retired.load(std::sync::atomic::Ordering::Relaxed) {
			return Err(error::SwapchainError::SwapchainRetired)
		}
		self.retired
			.store(true, std::sync::atomic::Ordering::Relaxed);

		let c_info = vk::SwapchainCreateInfoKHR::builder()
			.surface(**self.surface)
			.pre_transform(create_info.pre_transform)
			.composite_alpha(create_info.composite_alpha)
			.present_mode(create_info.present_mode)
			.clipped(create_info.clipped)
			.old_swapchain(*lock)
			.image_sharing_mode(create_info.sharing_mode.sharing_mode())
			.queue_family_indices(create_info.sharing_mode.indices());

		let c_info = create_info.image_info.add_to_create_info(c_info);

		unsafe {
			Self::from_create_info(
				self.device.clone(),
				self.surface.clone(),
				c_info,
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
			retired: AtomicVool::new(false),

			host_memory_allocator
		});

		let images: Vec<_> = me
			.loader
			.get_swapchain_images(swapchain)? // This is still okay since we haven't given anyone else access to the `swapchain` or `me` object, no synchronization problem
			.into_iter().enumerate()
			.map(|(index, image)| {
				image::SwapchainImage::new(
					me.clone(),
					Image::from_existing(
						device.clone(),
						image,
						None,
						c_info.image_usage,
						c_info.image_format,
						ImageSize::new_2d(
							NonZeroU32::new_unchecked(c_info.image_extent.width),
							NonZeroU32::new_unchecked(c_info.image_extent.height),
							NonZeroU32::new_unchecked(c_info.image_array_layers),
							MipmapLevels::One()
						).into(),
						HostMemoryAllocator::Unspecified()
					),
					index as u32
				)
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

	pub fn acquire_next(
		&self,
		timeout: crate::util::WaitTimeout,
		synchronization: AcquireSynchronization
	) -> error::AcquireResult {
		#[cfg(feature = "runtime_implicit_validations")]
		{
			if let Some(semaphore) = synchronization.semaphore() {
				if semaphore.device() != self.device() {
					return Err(error::AcquireError::SemaphoreSwapchainDeviceMismatch)
				}
			}
			if let Some(fence) = synchronization.fence() {
				if fence.device() != self.device() {
					return Err(error::AcquireError::FenceSwapchainDeviceMismatch)
				}
			}
		}

		let lock = self.swapchain.lock().expect("vutex poisoned");
		let semaphore_lock = synchronization
			.semaphore()
			.map(|f| f.lock().expect("vutex poisoned"));
		let fence_lock = synchronization
			.fence()
			.map(|f| f.lock().expect("vutex poisoned"));

		let result = unsafe {
			self.loader.acquire_next_image(
				*lock,
				timeout.into(),
				semaphore_lock
					.as_deref()
					.copied()
					.unwrap_or(vk::Semaphore::null()),
				fence_lock.as_deref().copied().unwrap_or(vk::Fence::null())
			)
		};

		match result {
			Ok((index, false)) => Ok(error::AcquireResultValue::SUCCESS(index)),
			Ok((index, true)) => Ok(error::AcquireResultValue::SUBOPTIMAL_KHR(index)),
			Err(e) => Err(e.into())
		}
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

	pub fn retired(&self) -> bool {
		self.retired.load(std::sync::atomic::Ordering::Relaxed)
	}
}
impl_common_handle_traits! {
	impl HasSynchronizedHandle<vk::SwapchainKHR>, Deref, Borrow, Eq, Hash, Ord for Swapchain {
		target = { swapchain }
	}
}
impl Drop for Swapchain {
	fn drop(&mut self) {
		let lock = self.swapchain.lock().expect("vutex poisoned");
		log_trace_common!("Dropping", self, lock);

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
