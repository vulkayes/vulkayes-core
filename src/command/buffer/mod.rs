use std::{fmt::Debug, ops::Deref};

use ash::vk;

use crate::{
	command::pool::CommandPool,
	prelude::{HasSynchronizedHandle, Vrc},
	util::sync::Vutex
};

use super::error::CommandBufferError;

pub mod recording;
// pub mod clear;
// pub mod control;
// pub mod render_pass;
// pub mod bind;

pub struct CommandBuffer {
	pool: Vrc<CommandPool>,
	command_buffer: Vutex<vk::CommandBuffer>
}
impl CommandBuffer {
	pub fn new(pool: Vrc<CommandPool>, primary: bool) -> Result<Vrc<Self>, CommandBufferError> {
		let level = match primary {
			true => vk::CommandBufferLevel::PRIMARY,
			false => vk::CommandBufferLevel::SECONDARY
		};
		let [raw] = unsafe { pool.allocate_command_buffer(level)? };

		Ok(Vrc::new(unsafe { Self::from_existing(pool, raw) }))
	}

	pub fn new_multiple(
		pool: Vrc<CommandPool>,
		primary: bool,
		count: std::num::NonZeroU32
	) -> Result<Vec<Vrc<Self>>, CommandBufferError> {
		let level = match primary {
			true => vk::CommandBufferLevel::PRIMARY,
			false => vk::CommandBufferLevel::SECONDARY
		};
		let raw = unsafe { pool.allocate_command_buffers(level, count)? };

		let buffers: Vec<_> = raw
			.into_iter()
			.map(|command_buffer| {
				Vrc::new(unsafe { Self::from_existing(pool.clone(), command_buffer) })
			})
			.collect();

		Ok(buffers)
	}

	/// Creates a new `CommandBuffer` from existing handle.
	///
	/// ### Safety
	///
	/// `command_buffer` must be valid handle allocated from `pool`.
	pub unsafe fn from_existing(pool: Vrc<CommandPool>, command_buffer: vk::CommandBuffer) -> Self {
		log_trace_common!(
			"Creating CommandBuffer from existing handle:",
			pool,
			crate::util::fmt::format_handle(command_buffer)
		);

		Self {
			pool,
			command_buffer: Vutex::new(command_buffer)
		}
	}

	/// ### Panic
	///
	/// This function will panic if the vutex cannot be locked.
	pub fn reset(&self, release_resource: bool) -> Result<(), CommandBufferError> {
		let handle = self.lock_handle();

		let flags = if release_resource {
			vk::CommandBufferResetFlags::RELEASE_RESOURCES
		} else {
			vk::CommandBufferResetFlags::empty()
		};

		log_trace_common!(
			"Resetting command buffer:",
			crate::util::fmt::format_handle(*handle),
			flags
		);
		unsafe {
			self.pool()
				.device()
				.reset_command_buffer(*handle, flags)
				.map_err(CommandBufferError::from)
		}
	}

	/// Equivalent to calling `CommandBufferRecordingLock::new(self)`
	///
	/// ### Panic
	///
	/// This function will panic if the pool or the buffer vutex cannot be locked.
	pub fn begin_recording(
		&self,
		info: recording::CommandBufferBeginInfo
	) -> Result<recording::CommandBufferRecordingLockOutsideRenderPass, CommandBufferError> {
		let lock = recording::common::CommandBufferRecordingLockCommon::new(self);

		recording::CommandBufferRecordingLockOutsideRenderPass::new(lock, info)
	}

	pub const fn pool(&self) -> &Vrc<CommandPool> {
		&self.pool
	}
}
impl_common_handle_traits! {
	impl HasSynchronizedHandle<vk::CommandBuffer>, Deref, Borrow, Eq, Hash, Ord for CommandBuffer {
		target = { command_buffer }
	}
}
impl Drop for CommandBuffer {
	fn drop(&mut self) {
		let lock = self.command_buffer.lock().expect("vutex poisoned");
		log_trace_common!("Dropping", self, lock);

		unsafe { self.pool.free_command_buffers([*lock]) }
	}
}
impl Debug for CommandBuffer {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct("CommandBuffer")
			.field("pool", &self.pool)
			.field("command_buffer", &self.command_buffer)
			.finish()
	}
}
