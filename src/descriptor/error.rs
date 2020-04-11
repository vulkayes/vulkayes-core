vk_result_error! {
	#[derive(Debug)]
	pub enum DescriptorSetLayoutError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}