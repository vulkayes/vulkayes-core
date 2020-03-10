pub use inner::*;

#[cfg(not(feature = "single_thread"))]
mod inner {
	pub type Vrc<T> = std::sync::Arc<T>;

	pub type Vutex<T> = std::sync::Mutex<T>;
}

#[cfg(feature = "single_thread")]
mod inner {
	use std::cell::{BorrowMutError, RefCell, RefMut};

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

		pub fn lock(&self) -> Result<RefMut<T>, BorrowMutError> {
			self.0.try_borrow_mut()
		}
	}
}
