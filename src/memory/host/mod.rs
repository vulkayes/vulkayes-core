use ash::vk::AllocationCallbacks;

#[cfg(feature = "rust_host_allocator")]
mod rust;

unsafe_enum_variants! {
	enum HostMemoryAllocatorInner {
		/// The Vulkan implementation-dependent allocator will be used.
		pub Unspecified,
		/// A custom allocator will be used.
		///
		/// ## Safety
		/// See the [Vulkan Spec](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/html/vkspec.html#VkAllocationCallbacks) for info on safety.
		{unsafe} pub Custom(AllocationCallbacks)
	} as pub HostMemoryAllocator
}
impl HostMemoryAllocator {
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
impl Into<Option<AllocationCallbacks>> for HostMemoryAllocator {
	fn into(self) -> Option<AllocationCallbacks> {
		match self.0 {
			HostMemoryAllocatorInner::Unspecified => None,
			HostMemoryAllocatorInner::Custom(callbacks) => Some(callbacks)
		}
	}
}
impl Default for HostMemoryAllocator {
	fn default() -> Self { HostMemoryAllocator::Unspecified() }
}
