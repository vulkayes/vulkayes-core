use ash::vk;

use crate::prelude::HasSynchronizedHandle;
use crate::util::sync::VutexGuard;

use super::CommandBuffer;

/// Wrapper around `VutexGuard` and `CommandBuffer` reference that provides safe command recording functions.
///
/// TODO: This struct is under construction
#[derive(Debug)]
pub struct CommandBufferRecordingLock<'a> {
	pub(super) lock: VutexGuard<'a, vk::CommandBuffer>,
	pub(super) buffer: &'a CommandBuffer
}
impl<'a> CommandBufferRecordingLock<'a> {
	/// ### Panic
	///
	/// This function will panic of the command buffer vutex cannot be locked.
	pub fn new(command_buffer: &'a CommandBuffer) -> Self {
		CommandBufferRecordingLock {
			lock: command_buffer.lock_handle(),
			buffer: command_buffer
		}
	}
}