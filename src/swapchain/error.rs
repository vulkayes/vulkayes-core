vk_result_error! {
	#[derive(Debug)]
	pub enum SwapchainError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_DEVICE_LOST,
			ERROR_SURFACE_LOST_KHR,
			ERROR_NATIVE_WINDOW_IN_USE_KHR,
			ERROR_INITIALIZATION_FAILED
		}

		#[error("Swapchain is retired and can no longer be used")]
		SwapchainRetired
	}
}
