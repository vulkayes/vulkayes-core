vk_result_error! {
	#[derive(Debug)]
	pub enum ShaderError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_INVALID_SHADER_NV
		}
	}
}
