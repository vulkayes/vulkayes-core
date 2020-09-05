use std::ops::Deref;

use ash::vk;

use crate::{prelude::HasSynchronizedHandle, util::sync::VutexGuard};

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
	/// This function will panic if the command buffer vutex cannot be locked.
	pub fn new(command_buffer: &'a CommandBuffer) -> Self {
		CommandBufferRecordingLock {
			lock: command_buffer.lock_handle(),
			buffer: command_buffer
		}
	}

	/// Returns a reference to the locked command buffer.
	///
	/// Attempting to lock it again will result in a deadlock.
	pub fn buffer(&self) -> &'a CommandBuffer {
		self.buffer
	}
}
impl_common_handle_traits! {
	impl HasHandle<vk::CommandBuffer>, Deref, Borrow, Eq, Hash, Ord for CommandBufferRecordingLock<'_> {
		target = { lock.deref() }
	}
}
