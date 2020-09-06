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
	/// ### Safety
	///
	/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateGraphicsPipelines.html>.
	pub unsafe fn from_create_info(
		device: Vrc<Device>,
		create_info: impl Deref<Target = vk::GraphicsPipelineCreateInfo>,
		host_memory_allocator: HostMemoryAllocator
	) -> Result<Vrc<Self>, GraphicsPipelineError> {
		if log::log_enabled!(log::Level::Trace) {
			let create_info = debugize_struct!(
				create_info;
				{
					stages: {
						stage: stage;
						module: module;
						name: p_name | { std::ffi::CStr::from_ptr(p_name) };
						specialization_info: *p_specialization_info;
					} from *[stage_count] p_stages;
					vertex_input_state: {
						vertex_binding_descriptions: *[vertex_binding_description_count] p_vertex_binding_descriptions;
						vertex_attribute_descriptions: *[vertex_attribute_description_count] p_vertex_attribute_descriptions;
					} from *p_vertex_input_state;
					viewport_state: {
						viewports: *[viewport_count] p_viewports;
						scissors: *[scissor_count] p_scissors;
					} from *p_viewport_state;
					rasterization_state: *p_rasterization_state;
					multisample_state: *p_multisample_state;
					depth_stencil_state: *p_depth_stencil_state;
					color_blend_state: {
						logic_op_enable: logic_op_enable;
						logic_op: logic_op;
						attachment_count: attachment_count;
						attachments: *p_attachments;
						blend_constants: blend_constants;
					} from *p_color_blend_state;
					dynamic_state: {
						dynamic_states: *[dynamic_state_count] p_dynamic_states;
					} from *p_dynamic_state;
					layout: layout;
					render_pass: render_pass;
					subpass: subpass;
				}
			);

			log_trace_common!(
				"Creating graphics pipeline:",
				device,
				create_info.stages,
				create_info.vertex_input_state,
				create_info.viewport_state,
				create_info.rasterization_state,
				create_info.multisample_state,
				create_info.depth_stencil_state,
				create_info.color_blend_state,
				create_info.dynamic_state,
				create_info.layout,
				create_info.render_pass,
				create_info.subpass,
				host_memory_allocator
			);
		}

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
