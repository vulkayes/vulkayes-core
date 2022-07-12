vk_result_error! {
	#[derive(Debug)]
	pub enum InstanceError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_INITIALIZATION_FAILED,
			ERROR_LAYER_NOT_PRESENT,
			ERROR_EXTENSION_NOT_PRESENT,
			ERROR_INCOMPATIBLE_DRIVER
		}

		// TODO: This causes an error
		#[error("Instance layer and/or extension strings could not be converted into CStr")]
		// #[error(transparent)]
		NulError(#[from] std::ffi::NulError)
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum PhysicalDeviceEnumerationError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_INITIALIZATION_FAILED
		}
	}
}
