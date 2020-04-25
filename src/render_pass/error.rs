vk_result_error! {
	#[derive(Debug)]
	pub enum RenderPassError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Subpasses must not be empty")]
		SubpassesEmpty,

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Source stage mask of subpass dependency must not be 0")]
		SrcStageMaskZero,

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Destination stage mask of subpass dependency must not be 0")]
		DstStageMaskZero,
	}
}

#[derive(Error, Debug)]
pub enum SubpassDescriptionError {
	#[cfg(feature = "runtime_implicit_validations")]
	#[error(
		"Number of resolve attachment references must match number of color attachment references"
	)]
	ResolveAttachmentsLengthMismatch
}
