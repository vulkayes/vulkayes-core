use std::{
	convert::TryFrom,
	fmt::{Display, Error, Formatter}
};

use ash::vk;

use crate::util::{fmt::VkVersion, string::VkSmallString};

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
pub struct InstanceLayerProperties {
	pub layer_name: VkSmallString,
	pub spec_version: VkVersion,
	pub implementation_version: VkVersion,
	pub description: VkSmallString
}
impl TryFrom<vk::LayerProperties> for InstanceLayerProperties {
	type Error = std::str::Utf8Error;

	fn try_from(value: vk::LayerProperties) -> Result<Self, Self::Error> {
		Ok(InstanceLayerProperties {
			layer_name: VkSmallString::try_from(value.layer_name)?,
			spec_version: VkVersion(value.spec_version),
			implementation_version: VkVersion(value.implementation_version),
			description: VkSmallString::try_from(value.layer_name)?
		})
	}
}
impl Display for InstanceLayerProperties {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(
			f,
			"{} {} (impl. {}): {}",
			self.layer_name, self.spec_version, self.implementation_version, self.description
		)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct InstanceExtensionProperties {
	pub extension_name: VkSmallString,
	pub spec_version: VkVersion
}
impl TryFrom<vk::ExtensionProperties> for InstanceExtensionProperties {
	type Error = std::str::Utf8Error;

	fn try_from(value: vk::ExtensionProperties) -> Result<Self, Self::Error> {
		Ok(InstanceExtensionProperties {
			extension_name: VkSmallString::try_from(value.extension_name)?,
			spec_version: VkVersion(value.spec_version)
		})
	}
}
impl Display for InstanceExtensionProperties {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(f, "{} {}", self.extension_name, self.spec_version)
	}
}
