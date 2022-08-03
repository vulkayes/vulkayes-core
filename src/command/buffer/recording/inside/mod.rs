impl<'a> super::CommandBufferRecordingLockInsideRenderPass<'a> {
	pub fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
		log_trace_common!(
			"Drawing:",
			crate::util::fmt::format_handle(self.handle()),
			vertex_count,
			instance_count,
			first_vertex,
			first_instance
		);
		unsafe {
			self.device().cmd_draw(
				self.handle(),
				vertex_count,
				instance_count,
				first_vertex,
				first_instance
			);
		}
	}
}
