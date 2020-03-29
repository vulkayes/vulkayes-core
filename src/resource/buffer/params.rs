use crate::memory::device::BufferMemoryAllocator;
use crate::memory::device::never::NeverDeviceAllocator;

#[derive(Debug)]
pub enum AllocatorParams<'a, A: BufferMemoryAllocator = NeverDeviceAllocator> {
	None,
	Some {
		allocator: &'a A,
		requirements: A::AllocationRequirements
	}
}
impl Default for AllocatorParams<'static> {
	fn default() -> Self {
		AllocatorParams::None
	}
}