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

vk_result_error! {
	#[derive(Debug)]
	pub enum SurfaceSupportError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_SURFACE_LOST_KHR
		}

		#[error("Queue family index is greater or equal to the number of queue families for given physical device")]
		QueueFamilyIndexOutOfBounds,
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum SurfaceQueryError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY,
			ERROR_SURFACE_LOST_KHR
		}
	}
}
