use std::{
	convert::TryFrom,
	fmt::{Debug, Display, Error, Formatter},
	ops::Deref,
	os::raw::c_char,
	str::Utf8Error
};

/// A utf8 encoded null-terminated string that is backed by a fixed-size array.
#[derive(Clone, Copy)]
pub struct VkSmallString {
	array: [c_char; Self::MAX_STRING_SIZE],
	len: usize
}
impl VkSmallString {
	// TODO: This should be the max of { MAX_PHYSICAL_DEVICE_NAME_SIZE, MAX_EXTENSION_NAME_SIZE, MAX_DESCRIPTION_SIZE, MAX_DRIVER_NAME_SIZE_KHR, MAX_DRIVER_INFO_SIZE_KHR }
	// 	Right now this seems to be the correct value
	pub const MAX_STRING_SIZE: usize = 256;

	/// Creates a new `VkSmallString` from an existing `c_char` buffer.
	///
	/// ### Safety
	///
	/// They array bytes must be valid unicode.
	pub unsafe fn from_c_string_unchecked(array: [c_char; Self::MAX_STRING_SIZE]) -> Self {
		VkSmallString {
			len: array
				.iter()
				.enumerate()
				.find(|(_, &byte)| byte == 0)
				.unwrap()
				.0,
			array
		}
	}
}
impl TryFrom<[c_char; Self::MAX_STRING_SIZE]> for VkSmallString {
	type Error = Utf8Error;

	fn try_from(array: [c_char; Self::MAX_STRING_SIZE]) -> Result<Self, Self::Error> {
		let len = array
			.iter()
			.enumerate()
			.find(|(_, &byte)| byte == 0)
			.unwrap()
			.0;

		unsafe {
			std::str::from_utf8(std::slice::from_raw_parts(array.as_ptr() as *const u8, len))?
		};

		Ok(VkSmallString { array, len })
	}
}
impl Deref for VkSmallString {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		unsafe {
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				self.array.as_ptr() as *const u8,
				self.len
			))
		}
	}
}
impl Debug for VkSmallString {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(f, "{}", self.deref())
	}
}
impl Display for VkSmallString {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(f, "{}", self.deref())
	}
}
