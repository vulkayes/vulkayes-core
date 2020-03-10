vk_result_error! {
	#[derive(Debug)]
	pub enum QueueSubmitError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_DEVICE_LOST
		}
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum QueueWaitError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_DEVICE_LOST
		}
	}
}
