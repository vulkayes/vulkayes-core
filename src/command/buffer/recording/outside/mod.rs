pub mod barrier;
pub mod copy;

impl<'a> super::CommandBufferRecordingLockOutsideRenderPass<'a> {
	pub fn dispatch(&self, group_count: [u32; 3]) {
		log_trace_common!(
			"Dispatch:",
			crate::util::fmt::format_handle(self.handle()),
			group_count
		);

		unsafe {
			self.device().cmd_dispatch(
				self.handle(),
				group_count[0],
				group_count[1],
				group_count[2]
			)
		}
	}

	pub fn dispatch_base(&self, base: [u32; 3], group_count: [u32; 3]) {
		log_trace_common!(
			"Dispatch base:",
			crate::util::fmt::format_handle(self.handle()),
			base,
			group_count
		);

		unsafe {
			self.device().cmd_dispatch_base(
				self.handle(),
				base[0], base[1], base[2],
				group_count[0], group_count[1], group_count[2]
			)
		}
	}
}
