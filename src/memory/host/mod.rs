use ash::vk::AllocationCallbacks;

#[cfg(feature = "rust_host_allocator")]
mod rust;

unsafe_enum_variants! {
	#[derive(Debug, Copy, Clone)]
	enum HostMemoryAllocatorInner {
		/// The Vulkan implementation-dependent allocator will be used.
		pub Unspecified => { None },
		/// A custom allocator will be used.
		///
		/// ## Safety
		///
		/// See <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/html/vkspec.html#VkAllocationCallbacks>.
		{unsafe} pub Custom { callbacks: AllocationCallbacks } => { Some(callbacks) }
	} as pub HostMemoryAllocator impl Into<Option<AllocationCallbacks>>
}
impl HostMemoryAllocator {
	pub fn as_ref(&self) -> Option<&AllocationCallbacks> {
		match self.0 {
			HostMemoryAllocatorInner::Unspecified => None,
			HostMemoryAllocatorInner::Custom { ref callbacks } => Some(callbacks)
		}
	}

	/// Rust GlobalAllocator will be used.
	#[cfg(feature = "rust_host_allocator")]
	#[allow(non_snake_case)]
	pub fn Rust() -> Self {
		unsafe {
			HostMemoryAllocator::Custom(
				AllocationCallbacks::builder()
					.pfn_allocation(Some(rust::RustHostMemoryAllocator::rust_alloc))
					.pfn_reallocation(Some(rust::RustHostMemoryAllocator::rust_realloc))
					.pfn_free(Some(rust::RustHostMemoryAllocator::rust_free))
					.pfn_internal_allocation(Some(
						rust::RustHostMemoryAllocator::rust_internal_allocation
					))
					.pfn_internal_free(Some(rust::RustHostMemoryAllocator::rust_internal_free))
					.build()
			)
		}
	}
}
impl Default for HostMemoryAllocator {
	fn default() -> Self {
		HostMemoryAllocator::Unspecified()
	}
}

unsafe impl Send for HostMemoryAllocator {}
unsafe impl Sync for HostMemoryAllocator {}
