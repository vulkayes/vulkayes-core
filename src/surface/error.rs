use thiserror::Error;

vk_result_error! {
	#[derive(Debug)]
	pub enum SurfaceError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_NATIVE_WINDOW_IN_USE_KHR
		}
	}
}