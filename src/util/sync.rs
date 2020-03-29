//! This module contains type aliases and wrappers to make switching `single_thread` feature seamless.

pub use inner::*;

// IDEA: Consider adding Vrc<T> = ManuallyDrop<T> as an unsafe alternative

#[cfg(not(feature = "single_thread"))]
mod inner {
	/// A type alias to `Arc`.
	pub type Vrc<T> = std::sync::Arc<T>;

	/// A type alias to `Mutex`.
	pub type Vutex<T> = std::sync::Mutex<T>;
	/// A type alias to `MutexGuard`.
	pub type VutexGuard<'a, T> = std::sync::MutexGuard<'a, T>;
	/// A type alias to `AtomicBool`.
	pub type AtomicVool = std::sync::atomic::AtomicBool;

	/// Marker trait for `Send + Sync`.
	pub trait VSendSync: Send + Sync {}
	impl<T> VSendSync for T where T: Send + Sync {}
}

#[cfg(feature = "single_thread")]
mod inner {
	use std::cell::{BorrowMutError, Cell, RefCell, RefMut};

	/// A type alias to `Rc`.
	pub type Vrc<T> = std::rc::Rc<T>;

	/// Type that is interface-compatible with `Mutex` to be used in single-threaded context.
	///
	/// This type is treated as "poisoned" when it is attempted to lock it twice.
	#[derive(Debug)]
	#[repr(transparent)]
	pub struct Vutex<T>(pub RefCell<T>);
	impl<T> Vutex<T> {
		pub const fn new(value: T) -> Self {
			Vutex(RefCell::new(value))
		}

		pub fn lock(&self) -> Result<VutexGuard<T>, BorrowMutError> {
			self.0.try_borrow_mut()
		}
	}
	/// Type that is `Deref`-compatible with `MutexGuard` in single-thread context.
	pub type VutexGuard<'a, T> = RefMut<'a, T>;

	/// A type that is interface-compatible with `AtomicBool` to be used in single-threaded context.
	pub struct AtomicVool(pub std::cell::Cell<bool>);
	impl AtomicVool {
		pub const fn new(value: bool) -> Self {
			AtomicVool(Cell::new(value))
		}

		pub fn load(&self, _: std::sync::atomic::Ordering) -> bool {
			self.0.get()
		}

		pub fn store(&self, value: bool, _: std::sync::atomic::Ordering) {
			self.0.set(value)
		}

		pub fn swap(&self, value: bool, _: std::sync::atomic::Ordering) -> bool {
			self.0.replace(value)
		}
	}

	/// Empty marker trait as a "single thread alternative" to `Send + Sync`.
	pub trait VSendSync {}
	impl<T> VSendSync for T {}
}

#[cfg(not(feature = "single_thread"))]
mod test {
	#[allow(unused_imports)] // It is actually used
	use crate::Vrc;

	macro_rules! test_send_sync {
		(
			$(
				$name: ident: $test_type: ty
			),+
		) => {
			$(
				#[test]
				fn $name() {
					fn accepts_send_sync(_any: impl Send + Sync) {}

					accepts_send_sync(
						std::mem::MaybeUninit::<$test_type>::uninit()
					);
				}
			)+
		}
	}

	// These are compile-time tests to check correct trait properties
	// These test "fail" when they don't compile
	test_send_sync!(
		instance_send_sync: Vrc<crate::instance::Instance>,
		device_send_sync: Vrc<crate::device::Device>,
		//
		swapchain_send_sync: Vrc<crate::swapchain::Swapchain>,
		queue_send_sync: Vrc<crate::queue::Queue>,
		fence_send_sync: Vrc<crate::sync::fence::Fence>,
		semaphore_send_sync: Vrc<crate::sync::semaphore::Semaphore>,
		binary_semaphore_send_sync: Vrc<crate::sync::semaphore::BinarySemaphore>,
		//
		swapchain_image_send_sync: Vrc<crate::swapchain::image::SwapchainImage>,
		image_send_sync: Vrc<crate::resource::image::Image>,
		image_view_send_sync: Vrc<crate::resource::image::view::ImageView>,
		buffer_send_sync: Vrc<crate::resource::buffer::Buffer>,
		buffer_view_send_sync: Vrc<crate::resource::buffer::view::BufferView>,
		command_pool_send_sync: Vrc<crate::command::pool::CommandPool>,
		command_buffer_send_sync: Vrc<crate::command::buffer::CommandBuffer>
	);
}
