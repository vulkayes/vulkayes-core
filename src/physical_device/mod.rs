//! A physical device represents a real physical device on the system.

use std::{
	convert::TryInto,
	fmt::{Debug, Error, Formatter},
	ops::Deref
};

use ash::{
	version::InstanceV1_0,
	vk::{
		self,
		Format,
		FormatProperties,
		ImageCreateFlags,
		ImageTiling,
		ImageType,
		ImageUsageFlags,
		PhysicalDeviceFeatures,
		QueueFamilyProperties
	}
};

use crate::{instance::Instance, Vrc};

pub mod enumerate;
#[cfg(test)]
pub mod test;

#[derive(Clone)]
pub struct PhysicalDevice {
	instance: Vrc<Instance>,
	physical_device: ash::vk::PhysicalDevice
}
impl PhysicalDevice {
	/// Creates a new `PhysicalDevice` wrapper type.
	///
	/// ### Safety
	///
	/// The `instance` must be the parent of the `physical_device`.
	pub unsafe fn from_existing(
		instance: crate::Vrc<Instance>,
		physical_device: ash::vk::PhysicalDevice
	) -> Self {
		log_trace_common!(
			"Creating PhysicalDevice from existing handle:",
			instance,
			crate::util::fmt::format_handle(physical_device)
		);

		PhysicalDevice {
			instance,
			physical_device
		}
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateDeviceExtensionProperties.html>.
	pub fn extensions_properties(
		&self
	) -> Result<
		impl ExactSizeIterator<Item = enumerate::DeviceExtensionProperties>,
		enumerate::EnumerateError
	> {
		let enumerator = unsafe {
			self.instance
				.enumerate_device_extension_properties(self.physical_device)?
				.into_iter()
				.map(|p| p.try_into().unwrap())
		};

		Ok(enumerator)
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceFormatProperties.html>.
	pub fn format_properties(&self, format: Format) -> FormatProperties {
		unsafe {
			self.instance
				.get_physical_device_format_properties(self.physical_device, format)
		}
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html>.
	pub fn image_format_properties(
		&self,
		format: Format,
		image_type: ImageType,
		tiling: ImageTiling,
		usage: ImageUsageFlags,
		flags: ImageCreateFlags
	) -> Result<ash::vk::ImageFormatProperties, enumerate::ImageFormatPropertiesError> {
		let properties = unsafe {
			self.instance.get_physical_device_image_format_properties(
				self.physical_device,
				format,
				image_type,
				tiling,
				usage,
				flags
			)?
		};

		Ok(properties)
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceMemoryProperties.html>.
	pub fn memory_properties(&self) -> enumerate::PhysicalDeviceMemoryProperties {
		unsafe {
			self.instance
				.get_physical_device_memory_properties(self.physical_device)
				.into()
		}
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceProperties.html>.
	pub fn properties(&self) -> enumerate::PhysicalDeviceProperties {
		unsafe {
			self.instance
				.get_physical_device_properties(self.physical_device)
				.try_into()
				.unwrap()
		}
	}

	/// Returns number of family queues supported by this physical device.
	pub fn queue_family_count(&self) -> std::num::NonZeroU32 {
		let mut queue_count: u32 = 0;

		unsafe {
			self.instance
				.fp_v1_0()
				.get_physical_device_queue_family_properties(
					self.physical_device,
					&mut queue_count,
					std::ptr::null_mut()
				);
		}

		std::num::NonZeroU32::new(queue_count as u32).unwrap()
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceQueueFamilyProperties.html>.
	pub fn queue_family_properties(&self) -> Vec<QueueFamilyProperties> {
		unsafe {
			self.instance
				.get_physical_device_queue_family_properties(self.physical_device)
		}
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceFeatures.html>.
	pub fn features(&self) -> PhysicalDeviceFeatures {
		unsafe {
			self.instance
				.get_physical_device_features(self.physical_device)
		}
	}

	pub const fn instance(&self) -> &Vrc<Instance> {
		&self.instance
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for PhysicalDevice {
		type Target = vk::PhysicalDevice { physical_device }
	}
}
impl Debug for PhysicalDevice {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		f.debug_struct("PhysicalDevice")
			.field(
				"physical_device",
				&crate::util::fmt::format_handle(self.physical_device)
			)
			.finish()
	}
}
