//! A surface represents a connection between Vulkan and a window.

use std::{
	fmt::{Debug, Formatter},
	ops::Deref
};

use ash::vk;

use crate::{
	prelude::Instance,
	prelude::HostMemoryAllocator,
	prelude::PhysicalDevice,
	prelude::Vrc,
	prelude::HasHandle
};

pub mod error;

pub struct Surface {
	instance: Vrc<Instance>,
	loader: ash::extensions::khr::Surface,
	surface: ash::vk::SurfaceKHR,

	host_memory_allocator: HostMemoryAllocator
}
impl Surface {
	/// Creates a new surface from an existing `vk::SurfaceKHR`.
	///
	/// ### Safety
	///
	/// `instance` must be a parent of `surface`.
	/// `surface` must be a valid surface handle for the whole lifetime of this object.
	pub unsafe fn from_existing(
		instance: Vrc<Instance>,
		surface: vk::SurfaceKHR,
		host_memory_allocator: HostMemoryAllocator
	) -> Self {
		let loader =
			ash::extensions::khr::Surface::new(instance.entry().deref(), instance.deref().deref());

		log_trace_common!(
			"Creating surface from existing handle:",
			instance,
			surface,
			host_memory_allocator
		);
		Surface {
			instance,
			loader,
			surface,
			host_memory_allocator
		}
	}

	/// Queries whether the given queue on the given physical device supports this surface.
	pub fn physical_device_surface_support(
		&self,
		physical_device: &PhysicalDevice,
		queue_family_index: u32
	) -> Result<bool, error::SurfaceSupportError> {
		if queue_family_index > physical_device.queue_family_count().get() {
			return Err(error::SurfaceSupportError::QueueFamilyIndexOutOfBounds)
		}

		let supported = unsafe {
			self.loader.get_physical_device_surface_support(
				*physical_device.deref(),
				queue_family_index,
				self.surface
			)?
		};

		Ok(supported)
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceSurfacePresentModesKHR.html>.
	pub fn physical_device_surface_present_modes(
		&self,
		physical_device: &PhysicalDevice
	) -> Result<Vec<vk::PresentModeKHR>, error::SurfaceQueryError> {
		let modes = unsafe {
			self.loader
				.get_physical_device_surface_present_modes(*physical_device.deref(), self.surface)?
		};

		Ok(modes)
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceSurfaceCapabilitiesKHR.html>.
	pub fn physical_device_surface_capabilities(
		&self,
		physical_device: &PhysicalDevice
	) -> Result<vk::SurfaceCapabilitiesKHR, error::SurfaceQueryError> {
		let capabilities = unsafe {
			self.loader
				.get_physical_device_surface_capabilities(*physical_device.deref(), self.surface)?
		};

		Ok(capabilities)
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceSurfaceFormatsKHR.html>.
	pub fn physical_device_surface_formats(
		&self,
		physical_device: &PhysicalDevice
	) -> Result<Vec<vk::SurfaceFormatKHR>, error::SurfaceQueryError> {
		let formats = unsafe {
			self.loader
				.get_physical_device_surface_formats(*physical_device.deref(), self.surface)?
		};

		Ok(formats)
	}

	pub const fn instance(&self) -> &Vrc<Instance> {
		&self.instance
	}

	pub const fn loader(&self) -> &ash::extensions::khr::Surface {
		&self.loader
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::SurfaceKHR>, Deref, Borrow, Eq, Hash, Ord for Surface {
		target = { surface }
	}
}
impl Drop for Surface {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.loader
				.destroy_surface(self.surface, self.host_memory_allocator.as_ref());
		}
	}
}
impl Debug for Surface {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.debug_struct("Surface")
			.field("instance", &self.instance)
			.field("loader", &"<ash::extensions::khr::Surface>")
			.field("surface", &self.safe_handle())
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
