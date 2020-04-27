use thiserror::Error;

vk_result_error! {
	#[derive(Debug)]
	pub enum DescriptorSetLayoutError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum DescriptorPoolError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_FRAGMENTATION_EXT
		}
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum DescriptorSetError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_FRAGMENTATION_EXT,
			ERROR_OUT_OF_POOL_MEMORY
		}

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("At least one descriptor set layout must be specified")]
		LayoutsEmpty,

		#[cfg(feature = "runtime_implicit_validations")]
		#[error("The descriptor pool and all descriptor layouts must come from the same device")]
		DescriptorPoolLayoutsDeviceMismatch,
	}
}


vk_result_error! {
	#[derive(Debug)]
	pub enum SamplerError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_TOO_MANY_OBJECTS
		}
	}
}

#[derive(Error, Debug)]
pub enum DescriptorImageInfoError {
	#[cfg(feature = "runtime_implicit_validations")]
	#[error("Sampler and image view must come from the same device")]
	SamplerImageViewDeviceMismatch
}

#[derive(Error, Debug)]
pub enum DescriptorInlineUniformBlockInfoError {
	#[cfg(feature = "runtime_implicit_validations")]
	#[error("Data must not be empty")]
	DataEmpty,

	#[cfg(feature = "runtime_implicit_validations")]
	#[error("Data size must be a multiple of four")]
	SizeNotMultipleOfFour
}

#[derive(Error, Debug)]
pub enum DescriptorSetWriteError {
	#[cfg(feature = "runtime_implicit_validations")]
	#[error("Descriptor count must not be zero")]
	ZeroCount
}
