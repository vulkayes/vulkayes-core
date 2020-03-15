pub use inner::*;

#[cfg(not(feature = "single_thread"))]
mod inner {
	/// A type alias to `Arc`.
	pub type Vrc<T> = std::sync::Arc<T>;

	/// A type alias to `Mutex`.
	pub type Vutex<T> = std::sync::Mutex<T>;
	pub type VutexGuard<'a, T> = std::sync::MutexGuard<'a, T>;
}

#[cfg(feature = "single_thread")]
mod inner {
	use std::cell::{BorrowMutError, RefCell, RefMut};

	/// A type alias to `Rc`.
	pub type Vrc<T> = std::rc::Rc<T>;

	/// Type that is interface-compatible with `Mutex` to be used in single-threaded context.
	///
	/// This type is treated as "poisoned" when it is attempted to lock it twice.
	#[derive(Debug)]
	#[repr(transparent)]
	pub struct Vutex<T>(RefCell<T>);
	impl<T> Vutex<T> {
		pub const fn new(value: T) -> Self {
			Vutex(RefCell::new(value))
		}

		pub fn lock(&self) -> Result<VutexGuard<T>, BorrowMutError> {
			self.0.try_borrow_mut()
		}
	}
	pub type VutexGuard<'a, T> = RefMut<'a, T>;
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
		queue_send_sync: Vrc<crate::queue::Queue>,
		fence_send_sync: Vrc<crate::sync::fence::Fence>,
		semaphore_send_sync: Vrc<crate::sync::semaphore::Semaphore>,
		binary_semaphore_send_sync: Vrc<crate::sync::semaphore::BinarySemaphore>,
		swapchain_send_sync: Vrc<crate::swapchain::Swapchain>,
		swapchain_image_send_sync: Vrc<crate::swapchain::SwapchainImage>,
		image_send_sync: Vrc<crate::resource::image::Image>,
		command_pool_send_sync: Vrc<crate::command::pool::CommandPool>,
		command_buffer_send_sync: Vrc<crate::command::buffer::CommandBuffer>
	);
}
