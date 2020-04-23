use thiserror::Error;

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