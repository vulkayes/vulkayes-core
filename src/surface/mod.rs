//! A surface represents a connection between Vulkan and a window.

use std::{
	fmt::{Debug, Formatter},
	ops::Deref
};

use ash::vk::{self, AllocationCallbacks};

use crate::{instance::Instance, physical_device::PhysicalDevice, Vrc};

pub mod error;

/// Inner surface to allow "dropping without window" from the public `Surface` object.
struct InnerSurface {
	instance: Vrc<Instance>,
	loader: ash::extensions::khr::Surface,
	surface: ash::vk::SurfaceKHR,

	allocation_callbacks: Option<AllocationCallbacks>
}
impl Drop for InnerSurface {
	fn drop(&mut self) {
		unsafe {
			self.loader.destroy_surface(self.surface, self.allocation_callbacks.as_ref());
		}
	}
}
impl Debug for InnerSurface {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.debug_struct("Surface")
			.field("instance", &self.instance)
			.field("loader", &"<ash::extensions::khr::Surface>")
			.field("surface", &crate::util::fmt::format_handle(self.surface))
			.field("allocation_callbacks", &self.allocation_callbacks)
			.finish()
	}
}
pub struct Surface<Window> {
	window: Window,
	inner: InnerSurface
}
impl<Window> Surface<Window> {
	/// Creates a new surface from an existing `ash::vk::SurfaceKHR`.
	///
	/// ### Safety
	///
	/// `instance` must be a parent of `surface`.
	/// `surface` must be a valid surface handle for the whole lifetime of this object.
	pub unsafe fn new(
		instance: Vrc<Instance>, window: Window, surface: ash::vk::SurfaceKHR,
		allocation_callbacks: Option<AllocationCallbacks>
	) -> Self {
		let loader =
			ash::extensions::khr::Surface::new(instance.entry().deref(), instance.deref().deref());

		let inner = InnerSurface { instance, loader, surface, allocation_callbacks };

		Surface { window, inner }
	}

	/// Queries whether the given queue on the given physical device supports this surface.
	pub fn physical_device_surface_support(
		&self, physical_device: &PhysicalDevice, queue_family_index: u32
	) -> Result<bool, error::SurfaceSupportError> {
		if queue_family_index > physical_device.queue_family_count().get() {
			return Err(error::SurfaceSupportError::QueueFamilyIndexOutOfBounds)
		}

		let supported = unsafe {
			self.inner.loader.get_physical_device_surface_support(
				*physical_device.deref(),
				queue_family_index,
				self.inner.surface
			)?
		};

		Ok(supported)
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceSurfacePresentModesKHR.html>.
	pub fn physical_device_surface_present_modes(
		&self, physical_device: &PhysicalDevice
	) -> Result<Vec<vk::PresentModeKHR>, error::SurfaceQueryError> {
		let modes = unsafe {
			self.inner.loader.get_physical_device_surface_present_modes(
				*physical_device.deref(),
				self.inner.surface
			)?
		};

		Ok(modes)
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceSurfaceCapabilitiesKHR.html>.
	pub fn physical_device_surface_capabilities(
		&self, physical_device: &PhysicalDevice
	) -> Result<vk::SurfaceCapabilitiesKHR, error::SurfaceQueryError> {
		let capabilities = unsafe {
			self.inner.loader.get_physical_device_surface_capabilities(
				*physical_device.deref(),
				self.inner.surface
			)?
		};

		Ok(capabilities)
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceSurfaceFormatsKHR.html>.
	pub fn physical_device_surface_formats(
		&self, physical_device: &PhysicalDevice
	) -> Result<Vec<vk::SurfaceFormatKHR>, error::SurfaceQueryError> {
		let formats = unsafe {
			self.inner
				.loader
				.get_physical_device_surface_formats(*physical_device.deref(), self.inner.surface)?
		};

		Ok(formats)
	}

	pub fn instance(&self) -> &Vrc<Instance> { &self.inner.instance }

	pub fn window(&self) -> &Window { &self.window }

	/// Drops `self` but returns the owned window.
	pub fn drop_without_window(self) -> Window { self.window }
}
impl<W> Deref for Surface<W> {
	type Target = ash::vk::SurfaceKHR;

	fn deref(&self) -> &Self::Target { &self.inner.surface }
}
impl<W: Debug> Debug for Surface<W> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.debug_struct("Surface").field("window", &self.window).field("inner", &self.inner).finish()
	}
}
