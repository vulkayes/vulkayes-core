//! This crate provides core components for the vulkayes project.
//!
//! ## Crate features:
//!
//! ### `rust_host_allocator`
//!
//! Adds `Rust()` constructor to `HostMemoryAllocator` that uses Rusts `std::alloc` methods.
//!
//! ### `single_thread`
//!
//! Replaces uses of `Arc<T>` and `Mutex<T>` (dubbed as `Vrc` and `Vutex` across the crate) with `Rc<T>` and plain `T`.
//!
//! ### `crypto_secure_hash`
//!
//! Uses `std::collections::{HashMap, HashSet}` instead of `rustc_hash::{FxHashMap, FxHashSet}` (dubbed as `VHashMap` and `VHashSet`)  across the crate.

// Export `ash` because all other component will use it.
pub use ash;

pub use util::sync::Vrc;

/// Non zero `1u32` constant to avoid unnecessary unsafe blocks in constant contexts.
pub const NONZEROU32_ONE: std::num::NonZeroU32 = unsafe { std::num::NonZeroU32::new_unchecked(1) };

// Macros used inside and outside of the crate.
#[macro_use]
pub mod util;

pub mod memory;

pub mod command;
pub mod device;
pub mod entry;
pub mod instance;
pub mod physical_device;
pub mod queue;
pub mod resource;
pub mod surface;
pub mod swapchain;

#[cfg(test)]
mod test {
	pub fn setup_testing_logger() {
		let logger = edwardium_logger::Logger::new(
			[edwardium_logger::targets::stderr::StderrTarget::new(
				log::Level::Trace
			)],
			std::time::Instant::now()
		);
		match logger.init_boxed() {
			Ok(_) => (),
			Err(_) => ()
		} // Purposely ignore the result as only the first test will set the logger successfully.
	}
}
