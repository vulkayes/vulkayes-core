use std::{
	alloc::Layout,
	ffi::c_void,
	mem::MaybeUninit,
	ptr::null_mut,
	sync::{Mutex, MutexGuard, Once}
};

use ash::vk::{InternalAllocationType, SystemAllocationScope};

static mut ALLOCATOR: MaybeUninit<Mutex<RustHostMemoryAllocator>> = MaybeUninit::uninit();
static ALLOCATOR_INIT: Once = Once::new();

pub(super) struct RustHostMemoryAllocator {
	ptr_map: crate::util::hash::VHashMap<*mut u8, std::alloc::Layout>
}
// This is safe because we are only hashing the `*mut u8`, not dereferencing it.
unsafe impl Send for RustHostMemoryAllocator {}
unsafe impl Sync for RustHostMemoryAllocator {}

impl RustHostMemoryAllocator {
	unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
		let ptr = std::alloc::alloc(layout);

		log::trace!(
			"Allocated {} bytes aligned to {} at {:p}",
			layout.size(),
			layout.align(),
			ptr
		);
		self.ptr_map.insert(ptr, layout);

		ptr
	}

	unsafe fn realloc(&mut self, ptr: *mut u8, new_size: usize) -> *mut u8 {
		match self.ptr_map.remove(&ptr) {
			None => unreachable!(),
			Some(old_layout) => {
				let new_ptr = std::alloc::realloc(ptr, old_layout, new_size);

				log::trace!(
					"Reallocated from {} to {} bytes aligned to {} from {:p} to {:p}",
					old_layout.size(),
					new_size,
					old_layout.align(),
					ptr,
					new_ptr
				);
				let new_layout = if new_ptr != null_mut() { Layout::from_size_align_unchecked(new_size, old_layout.align()) } else { old_layout };

				self.ptr_map.insert(new_ptr, new_layout);
				new_ptr
			}
		}
	}

	unsafe fn dealloc(&mut self, ptr: *mut u8) {
		if ptr == null_mut() {
			return
		}

		let layout = match self.ptr_map.remove(&ptr) {
			None => unreachable!(),
			Some(layout) => layout
		};

		std::alloc::dealloc(ptr, layout);
		log::trace!(
			"Deallocated {} bytes aligned at {} from {:p}",
			layout.size(),
			layout.align(),
			ptr
		);
	}

	fn lock_init_allocator() -> MutexGuard<'static, RustHostMemoryAllocator> {
		ALLOCATOR_INIT.call_once(|| unsafe {
			ALLOCATOR
				.as_mut_ptr()
				.write(Mutex::new(RustHostMemoryAllocator {
					ptr_map: Default::default()
				}));
		});

		unsafe { ALLOCATOR.as_ptr().as_ref().unwrap().lock().unwrap() }
	}

	pub(super) unsafe extern "system" fn rust_alloc(
		p_user_data: *mut c_void,
		size: usize,
		alignment: usize,
		allocation_scope: SystemAllocationScope
	) -> *mut c_void {
		let mut allocator = Self::lock_init_allocator();

		log::trace!(
			"rust_alloc({:p}, {}, {}, {:?})",
			p_user_data,
			size,
			alignment,
			allocation_scope
		);

		allocator.alloc(Layout::from_size_align_unchecked(
			size, alignment
		)) as *mut c_void
	}

	pub(super) unsafe extern "system" fn rust_realloc(
		p_user_data: *mut c_void,
		p_original: *mut c_void,
		size: usize,
		alignment: usize,
		allocation_scope: SystemAllocationScope
	) -> *mut c_void {
		let mut allocator = Self::lock_init_allocator();

		log::trace!(
			"rust_realloc({:p}, {:p}, {}, {}, {:?})",
			p_user_data,
			p_original,
			size,
			alignment,
			allocation_scope
		);

		let ptr = if p_original == std::ptr::null_mut() {
			allocator.alloc(Layout::from_size_align_unchecked(
				size, alignment
			))
		} else if size == 0 {
			allocator.dealloc(p_original as *mut u8);
			null_mut()
		} else {
			allocator.realloc(p_original as *mut u8, size)
		};

		ptr as *mut c_void
	}

	pub(super) unsafe extern "system" fn rust_free(p_user_data: *mut c_void, p_memory: *mut c_void) {
		let mut allocator = Self::lock_init_allocator();

		log::trace!(
			"rust_free({:p}, {:p})",
			p_user_data,
			p_memory
		);

		allocator.dealloc(p_memory as *mut u8);
	}

	pub(super) unsafe extern "system" fn rust_internal_allocation(
		p_user_data: *mut c_void,
		size: usize,
		allocation_type: InternalAllocationType,
		allocation_scope: SystemAllocationScope
	) {
		log::trace!(
			"rust_internal_allocation({:p}, {}, {:?}, {:?})",
			p_user_data,
			size,
			allocation_type,
			allocation_scope
		);
	}

	pub(super) unsafe extern "system" fn rust_internal_free(
		p_user_data: *mut c_void,
		size: usize,
		allocation_type: InternalAllocationType,
		allocation_scope: SystemAllocationScope
	) {
		log::trace!(
			"rust_internal_free({:p}, {}, {:?}, {:?})",
			p_user_data,
			size,
			allocation_type,
			allocation_scope
		);
	}
}
