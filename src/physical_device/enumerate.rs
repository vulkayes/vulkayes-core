use crate::util::VkSmallString;
use std::{
	convert::TryFrom,
	fmt::{Debug, Display, Error, Formatter}
};

use arrayvec::ArrayVec;

use crate::util::fmt::VkVersion;
use ash::vk::{
	MemoryHeap,
	MemoryType,
	PhysicalDeviceLimits,
	PhysicalDeviceSparseProperties,
	PhysicalDeviceType
};
use std::str::Utf8Error;
use thiserror::Error;

vk_result_error! {
	#[derive(Debug)]
	pub enum EnumerateError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceExtensionProperties {
	pub extension_name: VkSmallString,
	pub spec_version: VkVersion
}
impl TryFrom<ash::vk::ExtensionProperties> for DeviceExtensionProperties {
	type Error = std::str::Utf8Error;

	fn try_from(value: ash::vk::ExtensionProperties) -> Result<Self, Self::Error> {
		Ok(DeviceExtensionProperties {
			extension_name: VkSmallString::try_from(value.extension_name)?,
			spec_version: VkVersion(value.spec_version)
		})
	}
}
impl Display for DeviceExtensionProperties {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(f, "{} {}", self.extension_name, self.spec_version)
	}
}


vk_result_error! {
	#[derive(Debug)]
	pub enum ImageFormatPropertiesError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_FORMAT_NOT_SUPPORTED
		}
	}
}

#[derive(Debug, Clone)] // TODO: arrayvec isn't copy
pub struct PhysicalDeviceMemoryProperties {
	pub memory_types: ArrayVec<[MemoryType; 32]>,
	pub memory_heaps: ArrayVec<[MemoryHeap; 16]>
}
impl From<ash::vk::PhysicalDeviceMemoryProperties> for PhysicalDeviceMemoryProperties {
	fn from(value: ash::vk::PhysicalDeviceMemoryProperties) -> Self {
		let mut memory_types = ArrayVec::from(value.memory_types);
		unsafe {
			memory_types.set_len(value.memory_type_count as usize);
		}

		let mut memory_heaps = ArrayVec::from(value.memory_heaps);
		unsafe {
			memory_heaps.set_len(value.memory_heap_count as usize);
		}

		PhysicalDeviceMemoryProperties { memory_types, memory_heaps }
	}
}

pub struct PhysicalDeviceProperties {
	pub api_version: VkVersion,
	pub driver_version: VkVersion,
	pub vendor_id: u32,
	pub device_id: u32,
	pub device_type: PhysicalDeviceType,
	pub device_name: VkSmallString,
	pub pipeline_cache_uuid: [u8; 16],
	pub limits: PhysicalDeviceLimits,
	pub sparse_properties: PhysicalDeviceSparseProperties
}
impl TryFrom<ash::vk::PhysicalDeviceProperties> for PhysicalDeviceProperties {
	type Error = Utf8Error;

	fn try_from(value: ash::vk::PhysicalDeviceProperties) -> Result<Self, Self::Error> {
		Ok(PhysicalDeviceProperties {
			api_version: VkVersion(value.api_version),
			driver_version: VkVersion(value.driver_version),
			vendor_id: value.vendor_id,
			device_id: value.device_id,
			device_type: value.device_type,
			device_name: VkSmallString::try_from(value.device_name)?,
			pipeline_cache_uuid: value.pipeline_cache_uuid,
			limits: value.limits,
			sparse_properties: value.sparse_properties
		})
	}
}
impl Debug for PhysicalDeviceProperties {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		f.debug_struct("PhysicalDeviceProperties")
			.field("api_version", &self.api_version)
			.field("driver_version", &self.driver_version)
			.field("vendor_id", &format_args!("0x{:x}", self.vendor_id))
			.field("device_id", &format_args!("0x{:x}", self.device_id))
			.field("device_type", &self.device_type)
			.field("device_name", &self.device_name)
			.field("pipeline_cache_uuid", &crate::util::fmt::format_uuid(self.pipeline_cache_uuid))
			.field("limits", &self.limits)
			.field("sparse_properties", &self.sparse_properties)
			.finish()
	}
}
