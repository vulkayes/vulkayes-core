use std::{fmt::Debug, ops::Deref};

use ash::vk;

use crate::{command::pool::CommandPool, util::sync::Vutex, Vrc};

use super::error::CommandBufferError;

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

	pub const fn pool(&self) -> &Vrc<CommandPool> {
		&self.pool
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for CommandBuffer {
		type Target = Vutex<vk::CommandBuffer> { command_buffer }

		to_handle { .lock().expect("vutex poisoned").deref() }
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
