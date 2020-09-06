use ash::vk;

use crate::prelude::{Device, Vrc};

pub trait CommandBufferRecordingCommon {
	fn handle(&self) -> vk::CommandBuffer;

	fn pool_handle(&self) -> vk::CommandPool;

	fn device(&self) -> &Vrc<Device>;
}
