use ash::vk;

use ash::version::DeviceV1_0;

use crate::prelude::{
	CommandBuffer,
	Device,
	GraphicsPipeline,
	PipelineLayout,
	HasHandle,
	SafeHandle,
	HasSynchronizedHandle,
	Vrc,
	VutexGuard,
	Transparent,
	Buffer,
	PushConstantsTrait
};

/// Wrapper around `VutexGuard` and `CommandBuffer` reference that provides safe command recording functions.
#[derive(Debug)]
pub struct CommandBufferRecordingLockCommon<'a> {
	pub(super) lock: VutexGuard<'a, vk::CommandBuffer>,
	pub(super) pool_lock: VutexGuard<'a, vk::CommandPool>,
	pub(super) buffer: &'a CommandBuffer
}
impl<'a> CommandBufferRecordingLockCommon<'a> {
	/// ### Panic
	///
	/// This function will panic if the pool or the buffer vutex cannot be locked.
	pub fn new(command_buffer: &'a CommandBuffer) -> Self {
		let pool_lock = command_buffer.pool().lock_handle();
		let lock = command_buffer.lock_handle();

		CommandBufferRecordingLockCommon {
			pool_lock,
			lock,
			buffer: command_buffer
		}
	}

	pub(super) fn handle(&self) -> vk::CommandBuffer {
		*self.lock
	}

	pub(super) fn pool_handle(&self) -> vk::CommandPool {
		*self.pool_lock
	}

	pub(super) fn device(&self) -> &Vrc<Device> {
		self.buffer.pool().device()
	}
}
impl<'a> CommandBufferRecordingLockCommon<'a> {
	pub fn bind_graphics_pipeline(&self, pipeline: &GraphicsPipeline) {
		log_trace_common!(
			"Binding pipeline:",
			crate::util::fmt::format_handle(self.handle()),
			pipeline
		);
		unsafe {
			self.device().cmd_bind_pipeline(
				self.handle(),
				vk::PipelineBindPoint::GRAPHICS,
				pipeline.handle()
			)
		}
	}

	pub fn bind_descriptor_sets<'d>(
		&self,
		bind_point: vk::PipelineBindPoint,
		layout: &PipelineLayout,
		first_set: u32,
		descriptor_sets: impl AsRef<[SafeHandle<'d, vk::DescriptorSet>]>,
		dynamic_offsets: impl AsRef<[u32]>
	) {
		log_trace_common!(
			"Binding descriptor sets:",
			crate::util::fmt::format_handle(self.handle()),
			layout,
			first_set,
			descriptor_sets.as_ref(),
			dynamic_offsets.as_ref()
		);

		unsafe {
			self.device().cmd_bind_descriptor_sets(
				self.handle(),
				bind_point,
				layout.handle(),
				first_set,
				Transparent::transmute_slice(descriptor_sets.as_ref()),
				dynamic_offsets.as_ref()
			)
		}
	}

	pub fn push_constants<P: PushConstantsTrait>(
		&self,
		layout: &PipelineLayout,
		value: &P
	) {
		log_trace_common!(
			"Pushing constants:",
			crate::util::fmt::format_handle(self.handle()),
			P::STAGE_FLAGS,
			P::OFFSET_DIV_FOUR,
			P::SIZE_DIV_FOUR,
			value,
			value.as_bytes()
		);

		unsafe {
			self.device().cmd_push_constants(
				self.handle(),
				layout.handle(),
				P::STAGE_FLAGS,
				P::OFFSET_DIV_FOUR * 4,
				value.as_bytes()
			)
		}
	}

	pub fn bind_vertex_buffers<'b>(
		&self,
		first_binding: u32,
		buffers: impl AsRef<[SafeHandle<'b, vk::Buffer>]>,
		offsets: impl AsRef<[vk::DeviceSize]>
	) {
		log_trace_common!(
			"Binding vertex buffers:",
			crate::util::fmt::format_handle(self.handle()),
			first_binding,
			buffers.as_ref(),
			offsets.as_ref()
		);
		unsafe {
			self.device().cmd_bind_vertex_buffers(
				self.handle(),
				first_binding,
				Transparent::transmute_slice(buffers.as_ref()),
				offsets.as_ref()
			)
		}
	}

	pub fn bind_index_buffer(
		&self,
		buffer: &Buffer,
		offset: vk::DeviceSize,
		index_type: vk::IndexType
	) {
		log_trace_common!(
			"Binding index buffer:",
			crate::util::fmt::format_handle(self.handle()),
			buffer,
			offset,
			index_type
		);
		unsafe {
			self.device().cmd_bind_index_buffer(
				self.handle(),
				buffer.handle(),
				offset,
				index_type
			)
		}
	}
}
impl<'a> CommandBufferRecordingLockCommon<'a> {
	pub fn set_viewports(
		&self,
		first_viewport: u32,
		viewports: impl AsRef<[vk::Viewport]>
	) {
		log_trace_common!(
			"Setting viewports:",
			crate::util::fmt::format_handle(self.handle()),
			first_viewport,
			viewports.as_ref()
		);
		unsafe {
			self.device().cmd_set_viewport(
				self.handle(),
				first_viewport,
				viewports.as_ref()
			)
		}
	}
}