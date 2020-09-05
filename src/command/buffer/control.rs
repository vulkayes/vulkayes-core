use ash::{version::DeviceV1_0, vk};

use super::super::error::CommandBufferError;

use crate::prelude::{HasHandle, HasSynchronizedHandle};

#[derive(Debug)]
pub enum CommandBufferBeginInfo {
	/// The command buffer can only be submitted once before being reset.
	OneTime,
	/// The command buffer can be submitted multiple times before being reset.
	ManyTimes {
		/// The command buffer can be submitted multiple times at once.
		simultaneous: bool
	}
}
impl Into<vk::CommandBufferBeginInfoBuilder<'static>> for CommandBufferBeginInfo {
	fn into(self) -> vk::CommandBufferBeginInfoBuilder<'static> {
		let mut builder = vk::CommandBufferBeginInfo::builder();
		match self {
			CommandBufferBeginInfo::OneTime => {
				builder = builder.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
			}
			CommandBufferBeginInfo::ManyTimes { simultaneous } if simultaneous => {
				builder = builder.flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
			}
			_ => ()
		}

		builder
	}
}

impl<'a> super::recording::CommandBufferRecordingLock<'a> {
	pub fn reset(&self, release_resource: bool) -> Result<(), CommandBufferError> {
		let flags = if release_resource {
			vk::CommandBufferResetFlags::RELEASE_RESOURCES
		} else {
			vk::CommandBufferResetFlags::empty()
		};
		
		log_trace_common!(
			"Resetting command buffer:",
			crate::util::fmt::format_handle(self.handle()),
			flags
		);
		unsafe {
			self.buffer.pool().device().reset_command_buffer(
				self.handle(),
				flags
			).map_err(CommandBufferError::from)
		}
	}

	pub fn begin(
		&self,
		info: CommandBufferBeginInfo
	) -> Result<(), CommandBufferError> {
		#[allow(unused_variables)] // keeping mutex alive
		let pool_lock = self.buffer.pool().lock_handle();

		log_trace_common!(
			"Beginning command buffer:",
			crate::util::fmt::format_handle(self.handle()),
			info
		);

		let command_buffer_begin_info: vk::CommandBufferBeginInfoBuilder = info.into();
		unsafe {
			self.buffer.pool().device().begin_command_buffer(
				self.handle(),
				&command_buffer_begin_info
			).map_err(CommandBufferError::from)
		}
	}

	pub fn end(&self) -> Result<(), CommandBufferError> {
		#[allow(unused_variables)] // keeping mutex alive
		let pool_lock = self.buffer.pool().lock_handle();

		log_trace_common!(
			"Ending command buffer:",
			crate::util::fmt::format_handle(self.handle())
		);
		unsafe {
			self.buffer.pool().device().end_command_buffer(
				self.handle()
			).map_err(CommandBufferError::from)
		}
	}
}
