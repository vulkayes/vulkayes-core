use std::{fmt, ops::Deref};

use ash::vk;

use super::error::ComputePipelineError;
use crate::prelude::{Device, HasHandle, HostMemoryAllocator, Vrc};

pub struct ComputePipeline {
	device: Vrc<Device>,
	pipeline: vk::Pipeline,
	host_memory_allocator: HostMemoryAllocator
}
impl ComputePipeline {
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateComputePipelines.html>.
	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::ComputePipelineCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, ComputePipelineError> {
		if log::log_enabled!(log::Level::Trace) {
			log_trace_common!(
				"Creating compute pipeline:",
				device,
				create_info.flags,
				create_info.stage,
				create_info.layout,
				host_memory_allocator
			);
		}

		let pipeline = device
			.create_compute_pipelines(
				vk::PipelineCache::null(),
				&[*create_info.deref()],
				host_memory_allocator.as_ref()
			)
			.map_err(|e| e.1)?
			.into_iter()
			.next().unwrap()
		;
		let me = ComputePipeline {
			device,
			pipeline,
			host_memory_allocator
		};

		Ok(Vrc::new(me))
	}

	pub const fn device(&self) -> &Vrc<Device> {
		&self.device
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::Pipeline>, Deref, Borrow, Eq, Hash, Ord for ComputePipeline {
		target = { pipeline }
	}
}
impl Drop for ComputePipeline {
	fn drop(&mut self) {
		log_trace_common!("Dropping", self);

		unsafe {
			self.device.destroy_pipeline(
				self.pipeline,
				self.host_memory_allocator.as_ref()
			)
		}
	}
}
impl fmt::Debug for ComputePipeline {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("ComputePipeline")
			.field("device", &self.device)
			.field("pipeline", &self.safe_handle())
			.field(
				"host_memory_allocator",
				&self.host_memory_allocator
			)
			.finish()
	}
}
