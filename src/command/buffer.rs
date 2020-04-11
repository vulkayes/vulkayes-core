use std::{fmt::Debug, ops::Deref};

use ash::vk;

use crate::{command::pool::CommandPool, util::sync::Vutex, Vrc};

use super::error::CommandBufferError;

pub struct CommandBuffer {
	pool: Vrc<CommandPool>,
	command_buffer: Vutex<vk::CommandBuffer>
}
impl CommandBuffer {
	pub fn new(
		pool: Vrc<CommandPool>,
		level: vk::CommandBufferLevel,
		count: std::num::NonZeroU32
	) -> Result<Vec<Vrc<Self>>, CommandBufferError> {
		let raw = pool.allocate_command_buffers(level, count)?;

		let buffers: Vec<_> = raw
			.into_iter()
			.map(|command_buffer| {
				Vrc::new(CommandBuffer {
					pool: pool.clone(),
					command_buffer: Vutex::new(command_buffer)
				})
			})
			.collect();

		Ok(buffers)
	}

	pub const fn pool(&self) -> &Vrc<CommandPool> {
		&self.pool
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for CommandBuffer {
		type Target = Vutex<ash::vk::CommandBuffer> { command_buffer }

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

