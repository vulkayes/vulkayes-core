use std::convert::TryInto;

use ash::version::EntryV1_0;

pub mod enumerate;

pub struct Entry {
	entry: ash::Entry
}
impl AsRef<ash::Entry> for Entry {
	fn as_ref(&self) -> &ash::Entry {
		&self.entry
	}
}
impl Entry {
	pub fn new() -> Result<Self, ash::LoadingError> {
		Ok(
			Entry {
				entry: ash::Entry::new()?
			}
		)
	}

	pub fn instance_layers(&self) -> Result<
		impl ExactSizeIterator<Item = enumerate::InstanceLayerProperties>,
		enumerate::EnumerateError
	> {
		Ok(
			self.entry.enumerate_instance_layer_properties()?.into_iter().map(|v| v.try_into().unwrap())
		)
	}

	pub fn instance_extensions(&self) -> Result<
		impl ExactSizeIterator<Item = enumerate::InstanceExtensionProperties>,
		enumerate::EnumerateError
	> {
		Ok(
			self.entry.enumerate_instance_extension_properties()?.into_iter().map(|v| v.try_into().unwrap())
		)
	}
}

