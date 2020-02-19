use std::{
	convert::TryInto,
	fmt::{Debug, Error, Formatter},
	ops::Deref
};

use ash::version::InstanceV1_0;

use crate::instance::Instance;
use ash::vk::{
	Format,
	FormatProperties,
	ImageCreateFlags,
	ImageTiling,
	ImageType,
	ImageUsageFlags,
	PhysicalDeviceFeatures,
	QueueFamilyProperties
};

pub mod enumerate;

#[derive(Clone)]
pub struct PhysicalDevice {
	pub(crate) physical_device: ash::vk::PhysicalDevice,
	pub(crate) instance: crate::Vrc<Instance>
}
impl PhysicalDevice {
	pub fn extensions_properties(
		&self
	) -> Result<
		impl ExactSizeIterator<Item = enumerate::DeviceExtensionProperties>,
		enumerate::EnumerateError
	> {
		unsafe {
			Ok(self
				.instance
				.enumerate_device_extension_properties(self.physical_device)?
				.into_iter()
				.map(|p| p.try_into().unwrap()))
		}
	}

	pub fn physical_device_format_properties(&self, format: Format) -> FormatProperties {
		unsafe { self.instance.get_physical_device_format_properties(self.physical_device, format) }
	}

	pub fn physical_device_image_format_properties(
		&self, format: Format, image_type: ImageType, tiling: ImageTiling, usage: ImageUsageFlags,
		flags: ImageCreateFlags
	) -> Result<ash::vk::ImageFormatProperties, enumerate::ImageFormatPropertiesError> {
		unsafe {
			self.instance
				.get_physical_device_image_format_properties(
					self.physical_device,
					format,
					image_type,
					tiling,
					usage,
					flags
				)
				.map_err(From::from)
		}
	}

	pub fn physical_device_memory_properties(&self) -> enumerate::PhysicalDeviceMemoryProperties {
		unsafe { self.instance.get_physical_device_memory_properties(self.physical_device).into() }
	}

	pub fn physical_device_properties(&self) -> enumerate::PhysicalDeviceProperties {
		unsafe {
			self.instance.get_physical_device_properties(self.physical_device).try_into().unwrap()
		}
	}

	pub fn physical_device_queue_family_properties(&self) -> Vec<QueueFamilyProperties> {
		unsafe { self.instance.get_physical_device_queue_family_properties(self.physical_device) }
	}

	pub fn physical_device_features(&self) -> PhysicalDeviceFeatures {
		unsafe { self.instance.get_physical_device_features(self.physical_device) }
	}
}
impl Deref for PhysicalDevice {
	type Target = ash::vk::PhysicalDevice;

	fn deref(&self) -> &Self::Target { &self.physical_device }
}
impl Debug for PhysicalDevice {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		f.debug_struct("PhysicalDevice")
			.field("physical_device", &crate::util::fmt::format_handle(self.physical_device))
			.field("instance", &self.instance)
			.finish()
	}
}
