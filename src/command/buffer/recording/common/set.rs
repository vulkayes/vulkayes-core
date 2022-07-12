use ash::vk;

impl<'a> super::CommandBufferRecordingLockCommon<'a> {
	pub fn set_viewports(&self, first_viewport: u32, viewports: impl AsRef<[vk::Viewport]>) {
		log_trace_common!(
			"Setting viewports:",
			crate::util::fmt::format_handle(self.handle()),
			first_viewport,
			viewports.as_ref()
		);
		unsafe {
			self.device()
				.cmd_set_viewport(self.handle(), first_viewport, viewports.as_ref())
		}
	}
}
