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
		SwapchainRetired,

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Image usage must not be empty")]
		ImageUsageEmpty,
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum AcquireError {
		vk {
			TIMEOUT,
			NOT_READY,
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_DEVICE_LOST,
			ERROR_OUT_OF_DATE_KHR,
			ERROR_SURFACE_LOST_KHR,
			ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT
		}

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Semaphore and swapchain must come from the same device")]
		SemaphoreSwapchainDeviceMismatch,

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Fence and swapchain must come from the same device")]
		FenceSwapchainDeviceMismatch,
	}
}
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AcquireResultValue {
	SUCCESS(u32),
	SUBOPTIMAL_KHR(u32)
}
impl AcquireResultValue {
	pub fn index(&self) -> u32 {
		match *self {
			AcquireResultValue::SUCCESS(i) | AcquireResultValue::SUBOPTIMAL_KHR(i) => i
		}
	}
}
pub type AcquireResult = Result<AcquireResultValue, AcquireError>;
