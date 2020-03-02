//! Utilities and macros.

pub use string::VkSmallString;

#[macro_use]
pub mod macros;

pub mod fmt;
pub mod string;

pub enum SharingMode<A: AsRef<[u32]> = [u32; 2]> {
	Exclusive,
	Concurrent(A)
}
