vk_result_error! {
	#[derive(Debug)]
	pub enum DeviceError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_INITIALIZATION_FAILED,
			ERROR_EXTENSION_NOT_PRESENT,
			ERROR_FEATURE_NOT_PRESENT,
			ERROR_TOO_MANY_OBJECTS,
			ERROR_DEVICE_LOST
		}

		#[error("Device layer and/or extension strings could not be converted into CStr")]
		NulError(#[from] std::ffi::NulError),

		#[error("Queue create info array must contain at least one element")]
		QueuesEmpty,

		#[error("Queue create info `queue_priorities` array must contain at least one element")]
		QueuePrioritiesEmpty
	}
}
