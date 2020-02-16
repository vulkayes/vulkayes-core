use std::{
	convert::TryFrom,
	fmt::{Display, Error, Formatter}
};

use crate::util::VkSmallString;

vk_result_error! {
	pub enum EnumerateError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}


#[derive(Debug, Clone, Copy)]
pub struct InstanceLayerProperties {
	pub layer_name: VkSmallString,
	pub spec_version: u32,
	pub implementation_version: u32,
	pub description: VkSmallString
}
impl TryFrom<ash::vk::LayerProperties> for InstanceLayerProperties {
	type Error = std::str::Utf8Error;

	fn try_from(value: ash::vk::LayerProperties) -> Result<Self, Self::Error> {
		unsafe {
			Ok(InstanceLayerProperties {
				layer_name: VkSmallString::from_c_string_unchecked(value.layer_name),
				spec_version: value.spec_version,
				implementation_version: value.implementation_version,
				description: VkSmallString::from_c_string_unchecked(value.layer_name)
			})
		}
	}
}
impl Display for InstanceLayerProperties {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(
			f,
			"{} v{}.{}.{} (impl. v{}.{}.{}): {}",
			self.layer_name,
			ash::vk_version_major!(self.spec_version),
			ash::vk_version_minor!(self.spec_version),
			ash::vk_version_patch!(self.spec_version),
			ash::vk_version_major!(self.implementation_version),
			ash::vk_version_minor!(self.implementation_version),
			ash::vk_version_patch!(self.implementation_version),
			self.description
		)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct InstanceExtensionProperties {
	pub extension_name: VkSmallString,
	pub spec_version: u32
}
impl TryFrom<ash::vk::ExtensionProperties> for InstanceExtensionProperties {
	type Error = std::str::Utf8Error;

	fn try_from(value: ash::vk::ExtensionProperties) -> Result<Self, Self::Error> {
		unsafe {
			Ok(InstanceExtensionProperties {
				extension_name: VkSmallString::from_c_string_unchecked(value.extension_name),
				spec_version: value.spec_version
			})
		}
	}
}
impl Display for InstanceExtensionProperties {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(
			f,
			"{} v{}.{}.{}",
			self.extension_name,
			ash::vk_version_major!(self.spec_version),
			ash::vk_version_minor!(self.spec_version),
			ash::vk_version_patch!(self.spec_version)
		)
	}
}
