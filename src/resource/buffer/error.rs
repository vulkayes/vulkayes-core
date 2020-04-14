vk_result_error! {
	#[derive(Debug)]
	pub enum BufferError [AllocError] where [AllocError: std::error::Error + 'static] {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS
		}

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Usage flags must not be empty")]
		UsageEmpty,

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("The memory must be allocated from the same device")]
		MemoryDeviceMismatch,

		#[error("Allocation error produced by the allocator parameter")]
		AllocationError(AllocError),
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum BufferViewError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}
