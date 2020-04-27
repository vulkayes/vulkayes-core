use crate::memory::device::{allocator::BufferMemoryAllocator, never::NeverDeviceAllocator};

#[derive(Debug)]
pub enum BufferAllocatorParams<'a, A: BufferMemoryAllocator = NeverDeviceAllocator> {
	None,
	Some {
		allocator: &'a A,
		requirements: A::AllocationRequirements
	}
}
impl Default for BufferAllocatorParams<'static> {
	fn default() -> Self {
		BufferAllocatorParams::None
	}
}
