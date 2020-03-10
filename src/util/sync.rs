pub use inner::*;

#[cfg(not(feature = "single_thread"))]
mod inner {
	pub type Vrc<T> = std::sync::Arc<T>;

	pub type Vutex<T> = std::sync::Mutex<T>;
}

#[cfg(feature = "single_thread")]
mod inner {
	pub type Vrc<T> = std::rc::Rc<T>;

	/// Type is interface compatible with `Mutex` to be used in single-threaded context.
	#[derive(Debug)]
	#[repr(transparent)]
	pub struct Vutex<T>(T);
	impl<T> Vutex<T> {
		pub fn new(value: T) -> Self {
			Vutex(value)
		}

		pub fn lock(&self) -> Result<&T, std::convert::Infallible> {
			Ok(&self.0)
		}
	}
}
