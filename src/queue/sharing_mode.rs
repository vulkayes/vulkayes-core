use thiserror::Error;
use crate::queue::Queue;

#[derive(Debug, Clone)]
pub struct SharingMode<A: AsRef<[u32]> = [u32; 1]>(A);
impl SharingMode<[u32; 1]> {
	pub const fn one(queue: u32) -> Self {
		SharingMode([queue])
	}
}
impl<A: AsRef<[u32]>> SharingMode<A> {
	pub fn new(queues: A) -> Result<Self, SharingModeError> {
		let ref_queues = queues.as_ref();

		if ref_queues.len() == 0 {
			return Err(SharingModeError::ZeroQueues)
		}

		let duplicate = ref_queues.iter().enumerate().any(|(index, first)| {
			ref_queues.iter().skip(index + 1).any(|second| first == second)
		});

		if duplicate {
			return Err(SharingModeError::NotUnique)
		}

		Ok(
			SharingMode(queues)
		)
	}

	pub fn sharing_mode(&self) -> ash::vk::SharingMode {
		debug_assert_ne!(self.0.as_ref().len(), 0);

		if self.0.as_ref().len() == 1 {
			ash::vk::SharingMode::EXCLUSIVE
		} else {
			ash::vk::SharingMode::CONCURRENT
		}
	}

	pub fn indices(&self) -> &[u32] {
		self.0.as_ref()
	}
}
impl<'a> From<&'a super::Queue> for SharingMode<[u32; 1]> {
	fn from(queue: &'a Queue) -> Self {
		SharingMode::one(queue.queue_family_index())
	}
}

#[derive(Error, Debug)]
pub enum SharingModeError {
	#[error("All specified queue families must be unique")]
	NotUnique,

	#[error("Must specify at least one queue")]
	ZeroQueues
}