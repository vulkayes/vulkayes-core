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

vk_result_error! {
	#[derive(Debug)]
	pub enum PresentError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_DEVICE_LOST,
			ERROR_OUT_OF_DATE_KHR,
			ERROR_SURFACE_LOST_KHR,
			ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT
		}
	}
}
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum PresentResult {
	SUCCESS,
	SUBOPTIMAL_KHR
}
impl From<bool> for PresentResult {
	fn from(value: bool) -> Self {
		if value {
			PresentResult::SUBOPTIMAL_KHR
		} else {
			PresentResult::SUCCESS
		}
	}
}
