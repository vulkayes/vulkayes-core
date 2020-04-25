vk_result_error! {
	#[derive(Debug)]
	pub enum PipelineLayoutError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Stage flags field of push constant range must not be empty.")]
		StageFlagsEmpty,
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum GraphicsPipelineError {
		vk {
			ERROR_PIPELINE_COMPILE_REQUIRED_EXT,
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_INVALID_SHADER_NV
		}

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("Stage flags field of push constant range must not be empty.")]
		StageFlagsEmpty,
	}
}
