use std::ops::Deref;

use ash::vk;

use crate::prelude::{Framebuffer, HasHandle, RenderPass};

use super::CommandBufferError;

pub mod common;
pub mod inside;
pub mod outside;

pub use common::CommandBufferRecordingLockCommon;

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
impl From<CommandBufferBeginInfo> for vk::CommandBufferBeginInfoBuilder<'static> {
	fn from(value: CommandBufferBeginInfo) -> vk::CommandBufferBeginInfoBuilder<'static> {
		let mut builder = vk::CommandBufferBeginInfo::builder();
		match value {
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

/// Wrapper around `VutexGuard` and `CommandBuffer` reference that provides safe command recording functions.
///
/// TODO: This struct is under construction
///
/// ### Panic
///
/// This structure will panic on `drop` if an error occurs with the `end_command_buffer` command.
/// It is recommended to call `end` instead.
#[derive(Debug)]
pub struct CommandBufferRecordingLockOutsideRenderPass<'a>(CommandBufferRecordingLockCommon<'a>);
impl<'a> CommandBufferRecordingLockOutsideRenderPass<'a> {
	pub fn new(
		lock: CommandBufferRecordingLockCommon<'a>,
		info: CommandBufferBeginInfo
	) -> Result<Self, CommandBufferError> {
		log_trace_common!(
			"Beginning command buffer:",
			crate::util::fmt::format_handle(lock.handle()),
			info
		);

		let command_buffer_begin_info: vk::CommandBufferBeginInfoBuilder = info.into();
		unsafe {
			lock.device()
				.begin_command_buffer(lock.handle(), &command_buffer_begin_info)?;
		}

		Ok(CommandBufferRecordingLockOutsideRenderPass(lock))
	}
}
impl<'a> Deref for CommandBufferRecordingLockOutsideRenderPass<'a> {
	type Target = CommandBufferRecordingLockCommon<'a>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<'a> CommandBufferRecordingLockOutsideRenderPass<'a> {
	pub fn begin_render_pass(
		self,
		render_pass: &RenderPass,
		framebuffer: &Framebuffer,
		render_area: vk::Rect2D,
		clear_values: impl AsRef<[vk::ClearValue]>,
		contents_inline: bool
	) -> CommandBufferRecordingLockInsideRenderPass<'a> {
		let create_info = vk::RenderPassBeginInfo::builder()
			.render_pass(render_pass.handle())
			.framebuffer(framebuffer.handle())
			.render_area(render_area)
			.clear_values(clear_values.as_ref());

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
			self.device()
				.cmd_begin_render_pass(self.handle(), &create_info, contents);
		}

		CommandBufferRecordingLockInsideRenderPass(self)
	}

	/// Ends the recording.
	///
	/// ### Safety
	///
	/// Must only be called once.
	unsafe fn end_mut(&mut self) -> Result<(), CommandBufferError> {
		log_trace_common!(
			"Ending command buffer:",
			crate::util::fmt::format_handle(self.handle())
		);
		self.device()
			.end_command_buffer(self.handle())
			.map_err(CommandBufferError::from)
	}

	/// Ends the recording and returns the lock.
	pub fn end(self) -> Result<CommandBufferRecordingLockCommon<'a>, CommandBufferError> {
		// Prevent drop so we don't call `end_command_buffer` twice
		let mut dont_drop = std::mem::ManuallyDrop::new(self);

		// Need to call `end_mut` manually to return the result.
		let result = unsafe { dont_drop.end_mut() };

		// Move the lock out, this is safe because drop is prevented
		let lock = unsafe { std::ptr::read(&dont_drop.0) };

		match result {
			Ok(()) => Ok(lock),
			Err(err) => Err(err)
		}
	}
}
impl Drop for CommandBufferRecordingLockOutsideRenderPass<'_> {
	fn drop(&mut self) {
		unsafe { self.end_mut().expect("Could not end command buffer") }
	}
}

/// ### Panic
///
/// This structure will panic on `drop` if the inner `CommandBufferRecordingLockOutsideRenderPass` panics on drop.
/// It is recommended to call `end_render_pass` and retrieve the inner lock instead.
pub struct CommandBufferRecordingLockInsideRenderPass<'a>(
	CommandBufferRecordingLockOutsideRenderPass<'a>
);
impl<'a> Deref for CommandBufferRecordingLockInsideRenderPass<'a> {
	type Target = CommandBufferRecordingLockCommon<'a>;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}
impl<'a> CommandBufferRecordingLockInsideRenderPass<'a> {
	pub fn next_subpass(&self, contents_inline: bool) {
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
		unsafe { self.device().cmd_next_subpass(self.handle(), contents) }
	}

	/// ### Safety
	///
	/// Must only be called once.
	unsafe fn end_render_pass_mut(&mut self) {
		log_trace_common!(
			"Recording EndRenderPass:",
			crate::util::fmt::format_handle(self.handle())
		);
		self.device().cmd_end_render_pass(self.handle());
	}

	/// Consumes this struct, ends the render pass and returns the `CommandBufferRecordingLockOutsideRenderPass`.
	pub fn end_render_pass(self) -> CommandBufferRecordingLockOutsideRenderPass<'a> {
		// Prevent drop so we don't call `cmd_end_render_pass` twice.
		let mut dont_drop = std::mem::ManuallyDrop::new(self);

		// Need to call `end_render_pass_mut` manually to "drop"
		// Need to get the inner lock out to return it
		unsafe {
			dont_drop.end_render_pass_mut();

			// Safe because drop is prevented
			std::ptr::read(&dont_drop.0)
		}
	}
}
impl Drop for CommandBufferRecordingLockInsideRenderPass<'_> {
	fn drop(&mut self) {
		unsafe { self.end_render_pass_mut() }
	}
}
