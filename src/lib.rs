//! This crate provides core components for the vulkayes project.

// Export `ash` because all other component will use it.
pub use ash;
// Export `thiserror` because of the `vk_result_error` macro.
pub use thiserror;

// Export these fast hash-collections for other components to use.
// I found these to be fastest of { hashbrown, stdlib, fnv, fx } with a local benchmark, hopefully that's true for mostly everyone.
pub type FastHashMap<K, V> = rustc_hash::FxHashMap<K, V>;
pub type FastHashSet<V> = rustc_hash::FxHashSet<V>;

#[cfg(not(feature = "non_atomic_vrc"))]
pub type Vrc<T> = std::sync::Arc<T>;
#[cfg(feature = "non_atomic_vrc")]
pub type Vrc<T> = std::rc::Rc<T>;

// Macros used inside and outside of the crate.
#[macro_use]
pub mod util;

pub mod memory;

pub mod device;
pub mod entry;
pub mod instance;
pub mod physical_device;

#[cfg(test)]
mod test {
	pub fn setup_testing_logger() {
		let logger = edwardium_logger::Logger::new(
			[edwardium_logger::targets::stderr::StderrTarget::new(log::Level::Trace)],
			std::time::Instant::now()
		);
		match logger.init_boxed() {
			Ok(_) => (),
			Err(_) => ()
		} // Purposely ignore the result as only the first test will set the logger successfully.
	}
}
