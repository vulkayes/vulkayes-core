#[derive(Debug)]
pub struct MappedMemory {

}

vk_result_error! {
	#[derive(Debug)]
	pub enum MappingError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_MEMORY_MAP_FAILED
		}
	}
}