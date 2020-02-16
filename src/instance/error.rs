vk_result_error! {
	pub enum InstanceError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_INITIALIZATION_FAILED,
			ERROR_LAYER_NOT_PRESENT,
			ERROR_EXTENSION_NOT_PRESENT,
			ERROR_INCOMPATIBLE_DRIVER
		}

		#[error(display = "Instance load error.")]
		LoadError(Vec<&'static str>),

		#[error(display = "Instance creation info strings could not be converted into CStr.")]
		NulError(#[error(cause)] std::ffi::NulError)
	}
}

impl From<ash::InstanceError> for InstanceError {
	fn from(err: ash::InstanceError) -> Self {
		match err {
			ash::InstanceError::LoadError(v) => InstanceError::LoadError(v),
			ash::InstanceError::VkError(r) => r.into()
		}
	}
}
