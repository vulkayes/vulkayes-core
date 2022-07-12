use ash::vk;

vk_result_error! {
	#[derive(Debug)]
	pub enum QueueSubmitError {
		vk {
			NOT_READY,
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_DEVICE_LOST
		}

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Queue family of the command buffer and of the queue does not match")]
		QueueFamilyMismatch,

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Queue and fence must be from the same device")]
		QueueFenceDeviceMismatch,

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Wait stage flags must not be empty for any of the the waits")]
		WaitStagesEmpty,

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Wait semaphores, command buffers and signal semaphores must be from the same device")]
		WaitBufferSignalDeviceMismatch,
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

vk_result_error! {
	#[derive(Debug)]
	pub enum QueuePresentError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_DEVICE_LOST,
			ERROR_OUT_OF_DATE_KHR,
			ERROR_SURFACE_LOST_KHR,
			ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT
		}

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Swapchains element must contain at least one element")]
		SwapchainsEmpty,

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Swapchains and wait semaphores must come from the same instance")]
		SwapchainsSempahoredInstanceMismatch
	}
}
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum QueuePresentSuccess {
	SUCCESS,
	SUBOPTIMAL_KHR
}
impl From<bool> for QueuePresentSuccess {
	fn from(value: bool) -> Self {
		if value {
			QueuePresentSuccess::SUBOPTIMAL_KHR
		} else {
			QueuePresentSuccess::SUCCESS
		}
	}
}
pub fn match_queue_present_result(
	result: vk::Result
) -> Result<QueuePresentSuccess, QueuePresentError> {
	match result {
		vk::Result::SUCCESS => Ok(QueuePresentSuccess::SUCCESS),
		vk::Result::SUBOPTIMAL_KHR => Ok(QueuePresentSuccess::SUBOPTIMAL_KHR),
		err => Err(QueuePresentError::from(err))
	}
}
