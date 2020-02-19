use std::{
	convert::TryInto,
	fmt::{Debug, Error, Formatter}
};

use ash::version::EntryV1_0;
use std::ops::Deref;

pub mod enumerate;

pub struct Entry {
	entry: ash::Entry
}
impl Entry {
	pub fn new() -> Result<Self, ash::LoadingError> { Ok(Entry { entry: ash::Entry::new()? }) }

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

	pub fn instance_extensions(
		&self
	) -> Result<
		impl ExactSizeIterator<Item = enumerate::InstanceExtensionProperties>,
		enumerate::EnumerateError
	> {
		Ok(self
			.entry
			.enumerate_instance_extension_properties()?
			.into_iter()
			.map(|v| v.try_into().unwrap()))
	}
}
impl Deref for Entry {
	type Target = ash::Entry;

	fn deref(&self) -> &Self::Target { &self.entry }
}
impl Debug for Entry {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		f.debug_struct("Entry").field("entry", &"<ash::Entry>").finish()
	}
}
