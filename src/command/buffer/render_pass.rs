use ash::{version::DeviceV1_0, vk};

use crate::prelude::{HasHandle, HasSynchronizedHandle, RenderPass, Framebuffer};

impl<'a> super::recording::CommandBufferRecordingLock<'a> {
	pub fn begin_render_pass(
		&self,
		render_pass: &RenderPass,
		framebuffer: &Framebuffer,
		render_area: vk::Rect2D,
		clear_values: impl AsRef<[vk::ClearValue]>,
		contents_inline: bool
	) {
		#[allow(unused_variables)] // keeping mutex alive
		let pool_lock = self.buffer.pool().lock_handle();

		let create_info = vk::RenderPassBeginInfo::builder()
			.render_pass(render_pass.handle())
			.framebuffer(framebuffer.handle())
			.render_area(render_area)
			.clear_values(clear_values.as_ref())
		;

		let contents = if contents_inline {
			vk::SubpassContents::INLINE
		} else {
			vk::SubpassContents::SECONDARY_COMMAND_BUFFERS
		};

		log_trace_common!(
			"Recording BeginRenderPass:",
			crate::util::fmt::format_handle(self.handle()),
			render_pass,
			framebuffer,
			render_area,
			contents
		);
		unsafe {
			self.buffer.pool().device().cmd_begin_render_pass(
				self.handle(),
				&create_info,
				contents
			)
		}
	}

	pub fn next_subpass(
		&self,
		contents_inline: bool
	) {
		#[allow(unused_variables)] // keeping mutex alive
		let pool_lock = self.buffer.pool().lock_handle();

		let contents = if contents_inline {
			vk::SubpassContents::INLINE
		} else {
			vk::SubpassContents::SECONDARY_COMMAND_BUFFERS
		};

		log_trace_common!(
			"Recording NextSubpass:",
			crate::util::fmt::format_handle(self.handle()),
			contents
		);
		unsafe {
			self.buffer.pool().device().cmd_next_subpass(
				self.handle(),
				contents
			)
		}
	}

	pub fn end_render_pass(
		&self
	) {
		#[allow(unused_variables)] // keeping mutex alive
		let pool_lock = self.buffer.pool().lock_handle();

		log_trace_common!(
			"Recording EndRenderPass:",
			crate::util::fmt::format_handle(self.handle())
		);
		unsafe {
			self.buffer.pool().device().cmd_end_render_pass(
				self.handle()
			)
		}
	}
}
