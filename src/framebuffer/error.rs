vk_result_error! {
	#[derive(Debug)]
	pub enum FramebufferError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("The device render pass was created with must match with the device all attachments were created on")]
		RenderPassAttachmentsDeviceMismatch,
	}
}
