//! An Entry are the base loaded function pointers to Vulkan.

use std::{
	convert::TryInto,
	fmt::{Debug, Error, Formatter},
	ops::Deref
};

use crate::util::fmt::VkVersion;

pub mod enumerate;
#[cfg(test)]
pub mod test;

#[derive(Clone)]
pub struct Entry {
	entry: ash::Entry
}
impl Entry {
	pub fn new() -> Result<Self, ash::LoadingError> {
		Ok(Entry {
			entry: unsafe { ash::Entry::load()? }
		})
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceLayerProperties.html>.
	pub fn instance_layers(
		&self
	) -> Result<
		impl ExactSizeIterator<Item = enumerate::InstanceLayerProperties>,
		enumerate::EnumerateError
	> {
		Ok(self
			.entry
			.enumerate_instance_layer_properties()?
			.into_iter()
			.map(|v| v.try_into().unwrap()))
	}

	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceExtensionProperties.html>.
	pub fn instance_extensions(
		&self
	) -> Result<
		impl ExactSizeIterator<Item = enumerate::InstanceExtensionProperties>,
		enumerate::EnumerateError
	> {
		Ok(self
			.entry
			.enumerate_instance_extension_properties(None)?
			.into_iter()
			.map(|v| v.try_into().unwrap()))
	}

	pub fn instance_version(&self) -> VkVersion {
		match self.entry.try_enumerate_instance_version() {
			Ok(Some(v)) => VkVersion(v),
			Ok(None) => VkVersion::new(1, 0, 0),
			Err(err) => unreachable!("{}", err) // Should never happen as per Vulkan spec
		}
	}
}
impl Deref for Entry {
	type Target = ash::Entry;

	fn deref(&self) -> &Self::Target {
		&self.entry
	}
}
impl Debug for Entry {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		f.debug_struct("Entry")
			.field("entry", &"<ash::Entry>")
			.finish()
	}
}
