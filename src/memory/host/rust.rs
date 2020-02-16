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
	ptr_map: crate::FastHashMap<*mut u8, std::alloc::Layout>
}

unsafe impl Send for RustHostMemoryAllocator {}

unsafe impl Sync for RustHostMemoryAllocator {}

impl RustHostMemoryAllocator {
	unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
		let ptr = std::alloc::alloc(layout);

		log::trace!("Allocated {} bytes aligned to {} at {:p}", layout.size(), layout.align(), ptr);
		self.ptr_map.insert(ptr, layout);

		ptr
	}

	unsafe fn realloc(&mut self, ptr: *mut u8, new_size: usize) -> *mut u8 {
		match self.ptr_map.get_mut(&ptr) {
			None => unreachable!(),
			Some(old_layout) => {
				let new_ptr = std::alloc::realloc(ptr, *old_layout, new_size);

				log::trace!(
					"Reallocated from {} to {} bytes aligned to {} from {:p} to {:p}",
					old_layout.size(),
					new_size,
					old_layout.align(),
					ptr,
					new_ptr
				);
				if new_ptr != null_mut() {
					*old_layout = Layout::from_size_align_unchecked(new_size, old_layout.align());
				}

				new_ptr
			}
		}
	}

	unsafe fn dealloc(&mut self, ptr: *mut u8) {
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
				.write(Mutex::new(RustHostMemoryAllocator { ptr_map: Default::default() }));
		});

		unsafe { ALLOCATOR.as_ptr().as_ref().unwrap().lock().unwrap() }
	}

	pub(super) unsafe extern "system" fn rust_alloc(
		_: *mut c_void, size: usize, alignment: usize, _: SystemAllocationScope
	) -> *mut c_void {
		let mut allocator = Self::lock_init_allocator();

		allocator.alloc(Layout::from_size_align_unchecked(size, alignment)) as *mut c_void
	}

	pub(super) unsafe extern "system" fn rust_realloc(
		_: *mut c_void, p_original: *mut c_void, size: usize, alignment: usize,
		_: SystemAllocationScope
	) -> *mut c_void {
		let mut allocator = Self::lock_init_allocator();

		let ptr = if p_original == std::ptr::null_mut() {
			allocator.alloc(Layout::from_size_align_unchecked(size, alignment))
		} else if size == 0 {
			allocator.dealloc(p_original as *mut u8);
			null_mut()
		} else {
			allocator.realloc(p_original as *mut u8, size)
		};

		ptr as *mut c_void
	}

	pub(super) unsafe extern "system" fn rust_free(
		_: *mut c_void, p_memory: *mut c_void
	) -> c_void {
		let mut allocator = Self::lock_init_allocator();

		allocator.dealloc(p_memory as *mut u8);

		std::mem::MaybeUninit::uninit().assume_init()
	}

	pub(super) unsafe extern "system" fn rust_internal_allocation(
		p_user_data: *mut c_void, size: usize, allocation_type: InternalAllocationType,
		allocation_scope: SystemAllocationScope
	) -> c_void {
		log::trace!(
			"rust_internal_allocation({:p}, {}, {:?}), {:?}",
			p_user_data,
			size,
			allocation_type,
			allocation_scope
		);

		std::mem::MaybeUninit::uninit().assume_init()
	}

	pub(super) unsafe extern "system" fn rust_internal_free(
		p_user_data: *mut c_void, size: usize, allocation_type: InternalAllocationType,
		allocation_scope: SystemAllocationScope
	) -> c_void {
		log::trace!(
			"rust_internal_free({:p}, {}, {:?}), {:?}",
			p_user_data,
			size,
			allocation_type,
			allocation_scope
		);

		std::mem::MaybeUninit::uninit().assume_init()
	}
}
