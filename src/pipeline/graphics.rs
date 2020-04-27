use std::{fmt, ops::Deref};

use ash::{version::DeviceV1_0, vk};

use crate::prelude::{Device, HasHandle, HostMemoryAllocator, Vrc};

use super::error::GraphicsPipelineError;

pub struct GraphicsPipeline {
	device: Vrc<Device>,
	pipeline: vk::Pipeline,
	host_memory_allocator: HostMemoryAllocator
}
impl GraphicsPipeline {
	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::GraphicsPipelineCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, GraphicsPipelineError> {
		log_trace_common!(
			"Creating graphics pipeline:",
			device,
			create_info.deref(),
			host_memory_allocator
		);

		let pipeline = device
			.create_graphics_pipelines(
				vk::PipelineCache::null(),
				&[*create_info.deref()],
				host_memory_allocator.as_ref()
			)
			.map_err(|e| e.1)?
			.into_iter()
			.next()
			.unwrap();

		Ok(Vrc::new(GraphicsPipeline {
			device,
			pipeline,
			host_memory_allocator
		}))
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::Pipeline>, Deref, Borrow, Eq, Hash, Ord for GraphicsPipeline {
		target = { pipeline }
	}
}
impl Drop for GraphicsPipeline {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.device
				.destroy_pipeline(self.pipeline, self.host_memory_allocator.as_ref())
		}
	}
}
impl fmt::Debug for GraphicsPipeline {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("GraphicsPipeline")
			.field("device", &self.device)
			.field("pipeline", &self.safe_handle())
			.field("host_memory_allocator", &self.host_memory_allocator)
			.finish()
	}
}
