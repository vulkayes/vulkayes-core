use ash::vk;
use ash::version::DeviceV1_0;

use crate::{prelude::{HasHandle, HasSynchronizedHandle, Device, Vrc, RenderPass, Framebuffer}, util::sync::VutexGuard};

use super::{CommandBuffer, CommandBufferError};

pub mod common;
// pub mod inside_render_pass;
// pub mod outside_render_pass;

pub use common::CommandBufferRecordingCommon;

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

/// Wrapper around `VutexGuard` and `CommandBuffer` reference that provides safe command recording functions.
///
/// TODO: This struct is under construction
///
/// ### Panic
///
/// This structure will panic on `drop` if an error occurs with the `end_command_buffer` command.
/// It is recommended to call `end` instead.
#[derive(Debug)]
pub struct CommandBufferRecordingLock<'a> {
	pub(super) lock: VutexGuard<'a, vk::CommandBuffer>,
	pub(super) pool_lock: VutexGuard<'a, vk::CommandPool>,
	pub(super) buffer: &'a CommandBuffer
}
impl<'a> CommandBufferRecordingLock<'a> {
	/// ### Panic
	///
	/// This function will panic if the pool or the buffer vutex cannot be locked.
	pub fn new(
		command_buffer: &'a CommandBuffer,
		info: CommandBufferBeginInfo
	) -> Result<Self, CommandBufferError> {
		let pool_lock = command_buffer.pool().lock_handle();
		let lock = command_buffer.lock_handle();

		log_trace_common!(
			"Beginning command buffer:",
			crate::util::fmt::format_handle(*lock),
			info
		);

		let command_buffer_begin_info: vk::CommandBufferBeginInfoBuilder = info.into();
		unsafe {
			command_buffer.pool().device().begin_command_buffer(
				*lock,
				&command_buffer_begin_info
			)?;
		}

		Ok(
			CommandBufferRecordingLock {
				pool_lock,
				lock,
				buffer: command_buffer
			}
		)
	}

	/// Returns a reference to the locked command buffer.
	///
	/// Attempting to lock it again will result in a deadlock.
	pub fn buffer(&self) -> &'a CommandBuffer {
		self.buffer
	}
}
impl CommandBufferRecordingCommon for CommandBufferRecordingLock<'_> {
	fn handle(&self) -> vk::CommandBuffer {
		*self.lock
	}

	fn pool_handle(&self) -> vk::CommandPool {
		*self.pool_lock
	}

	fn device(&self) -> &Vrc<Device> {
		self.buffer.pool().device()
	}
}
impl<'a> CommandBufferRecordingLock<'a> {
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
			self.device().cmd_begin_render_pass(
				self.handle(),
				&create_info,
				contents
			);
		}

		CommandBufferRecordingLockInsideRenderPass {
			inner: Some(self)
		}
	}

	/// This consumes the recording object, dropping the held locks and ending the recording.
	///
	/// ### Safety
	///
	/// Must only be called once.
	unsafe fn end_mut(&mut self) -> Result<(), CommandBufferError> {
		log_trace_common!(
			"Ending command buffer:",
			crate::util::fmt::format_handle(self.handle())
		);
		self.device().end_command_buffer(
			self.handle()
		).map_err(CommandBufferError::from)
	}

	pub fn end(self) -> Result<(), CommandBufferError> {
		// Prevent dropping because that would call `end_command_buffer` twice!
		// We need to call `end_mut` manually to return the result.
		let mut dont_drop = std::mem::ManuallyDrop::new(self);
		
		unsafe { dont_drop.end_mut() }
	}
}
impl Drop for CommandBufferRecordingLock<'_> {
	fn drop(&mut self) {
		unsafe { self.end_mut().expect("Could not end command buffer") }
	}
}

/// ### Panic
///
/// This structure will panic on `drop` if the inner `CommandBufferRecordingLock` panics on drop.
/// It is recommended to call `end_render_pass` and retrieve the inner lock instead.
pub struct CommandBufferRecordingLockInsideRenderPass<'a> {
	inner: Option<CommandBufferRecordingLock<'a>>
}
impl<'a> CommandBufferRecordingCommon for CommandBufferRecordingLockInsideRenderPass<'_> {
	fn handle(&self) -> vk::CommandBuffer {
		self.inner.as_ref().unwrap().handle()
	}

	fn pool_handle(&self) -> vk::CommandPool {
		self.inner.as_ref().unwrap().pool_handle()
	}

	fn device(&self) -> &Vrc<Device> {
		self.inner.as_ref().unwrap().device()
	}
}
impl<'a> CommandBufferRecordingLockInsideRenderPass<'a> {
	pub fn next_subpass(
		&self,
		contents_inline: bool
	) {
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
			self.device().cmd_next_subpass(
				self.handle(),
				contents
			)
		}
	}

	/// ### Safety
	///
	/// Must only be called once.
	unsafe fn end_render_pass_mut(
		&mut self
	) {
		log_trace_common!(
			"Recording EndRenderPass:",
			crate::util::fmt::format_handle(self.handle())
		);
		self.device().cmd_end_render_pass(
			self.handle()
		);
	}

	pub fn end_render_pass(
		mut self
	) -> CommandBufferRecordingLock<'a> {
		self.inner.take().unwrap()
		// Drop takes care of ending the render pass
	}
}
impl Drop for CommandBufferRecordingLockInsideRenderPass<'_> {
	fn drop(&mut self) {
		unsafe { self.end_render_pass_mut() }
	}
}