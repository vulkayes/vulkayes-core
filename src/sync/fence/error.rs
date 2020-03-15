vk_result_error! {
	#[derive(Debug)]
	pub enum FenceError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum FenceStatusError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_DEVICE_LOST
		}
	}
}