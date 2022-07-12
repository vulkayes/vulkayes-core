use ash::vk;

use crate::prelude::{CommandBuffer, Device, HasSynchronizedHandle, Vrc, VutexGuard};

pub mod bind;
pub mod set;

/// Wrapper around `VutexGuard` and `CommandBuffer` reference that provides safe command recording functions.
#[derive(Debug)]
pub struct CommandBufferRecordingLockCommon<'a> {
	pub(super) lock: VutexGuard<'a, vk::CommandBuffer>,
	pub(super) pool_lock: VutexGuard<'a, vk::CommandPool>,
	pub(super) buffer: &'a CommandBuffer
}
impl<'a> CommandBufferRecordingLockCommon<'a> {
	/// ### Panic
	///
	/// This function will panic if the pool or the buffer vutex cannot be locked.
	pub fn new(command_buffer: &'a CommandBuffer) -> Self {
		let pool_lock = command_buffer.pool().lock_handle();
		let lock = command_buffer.lock_handle();

		CommandBufferRecordingLockCommon { pool_lock, lock, buffer: command_buffer }
	}

	pub(super) fn handle(&self) -> vk::CommandBuffer {
		*self.lock
	}

	// pub(super) fn pool_handle(&self) -> vk::CommandPool {
	// 	*self.pool_lock
	// }

	pub(super) fn device(&self) -> &Vrc<Device> {
		self.buffer.pool().device()
	}
}
