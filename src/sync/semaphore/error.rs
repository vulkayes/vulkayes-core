vk_result_error! {
	#[derive(Debug)]
	pub enum SemaphoreError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}