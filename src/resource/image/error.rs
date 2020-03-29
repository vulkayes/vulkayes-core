vk_result_error! {
	#[derive(Debug)]
	pub enum ImageError [AllocError] where [AllocError: std::error::Error + 'static] {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}

		#[error("Usage flags must not be empty")]
		UsageEmpty,

		#[error("The memory must be allocated from the same device")]
		MemoryDeviceMismatch,

		#[error("Allocation error produced by the allocator parameter")]
		AllocationError(AllocError),
	}
}


vk_result_error! {
	#[derive(Debug)]
	pub enum ImageViewError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}
