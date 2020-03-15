vk_result_error! {
	#[derive(Debug)]
	pub enum QueueSubmitError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_DEVICE_LOST
		}

		#[error("Queue family of the command buffer and of the queue does not match")]
		QueueFamilyMismatch,

		#[error("Queue and fence must be from the same device")]
		QueueFenceDeviceMismatch,

		#[error("Wait stage flags must not be empty for any of the the waits")]
		WaitStagesEmpty,

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
