vk_result_error! {
	#[derive(Debug)]
	pub enum CommandPoolError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum CommandBufferError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}