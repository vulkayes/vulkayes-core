use std::ffi::NulError;
//#[derive(Error)]
pub enum InstanceCreationError {}

impl From<ash::InstanceError> for InstanceCreationError {
	fn from(err: ash::InstanceError) -> Self {
		unimplemented!()
	}
}
impl From<ash::vk::Result> for InstanceCreationError {
	fn from(err: ash::vk::Result) -> Self {
		unimplemented!()
	}
}
impl From<std::ffi::NulError> for InstanceCreationError {
	fn from(err: NulError) -> Self {
		unimplemented!()
	}
}