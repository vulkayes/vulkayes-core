use std::{fmt::Debug, ops::Deref};

use ash::vk;

use crate::{command::pool::CommandPool, Vrc};

pub struct CommandBuffer {
	pool: Vrc<CommandPool>,
	command_buffer: vk::CommandBuffer
}
impl CommandBuffer {
	pub fn new(
		pool: Vrc<CommandPool>,
		level: vk::CommandBufferLevel,
		count: std::num::NonZeroU32
	) -> Result<Vec<Vrc<Self>>, CommandBufferError> {
		let raw = unsafe { pool.allocate_command_buffers(level, count)? };

		let buffers: Vec<_> = raw
			.into_iter()
			.map(|command_buffer| {
				Vrc::new(CommandBuffer {
					pool: pool.clone(),
					command_buffer
				})
			})
			.collect();

		Ok(buffers)
	}
}
impl_common_handle_traits! {
	impl Deref, PartialEq, Eq, Hash for CommandBuffer {
		type Target = ash::vk::CommandBuffer { command_buffer }
	}
}
impl Drop for CommandBuffer {
	fn drop(&mut self) {
		unsafe { self.pool.free_command_buffers([self.command_buffer]) }
	}
}
impl Debug for CommandBuffer {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct("CommandBuffer").finish()
	}
}

vk_result_error! {
	#[derive(Debug)]
	pub enum CommandBufferError {
		vk {
			ERROR_OUT_OF_HOST_MEMORY,
			ERROR_OUT_OF_DEVICE_MEMORY
		}
	}
}
