use std::{
	convert::TryFrom,
	fmt::{Debug, Display, Error, Formatter},
	ops::Deref,
	os::raw::c_char,
	str::Utf8Error
};

/// Generates a private enum and a public newtype struct that only has that enum as a value.
/// Also generated constructors on the struct that match the enum variants.
///
/// This is useful for making certain enum variants "unsafe" by only allowing their construction using
/// an unsafe function. Variants also may be private.
///
/// Structure enum variants are not supported.
#[macro_export]
macro_rules! unsafe_enum_variants {
	(
		$(#[$attribute: meta])*
		enum $inner_name: ident {
			$(
				$(#[$variant_attribute: meta])*
				$({$safety: tt})? $v: vis $variant: ident $( ($( $variant_data: ident ),+) )?
			),+
		} as pub $name: ident
	) => {
		enum $inner_name {
			$(
				$variant $( ($( $variant_data ),+) )?
			),+
		}
		$(#[$attribute])*
		pub struct $name($inner_name);
		impl $name {
			$(
				$(#[$variant_attribute])*
				#[allow(non_snake_case)]
				$v $($safety)? fn $variant($( $( $variant_data: $variant_data ),+ )?) -> Self {
					$name(
						$inner_name::$variant $( ($( $variant_data ),+) )?
					)
				}
			)*
		}
	}
}

#[macro_export]
macro_rules! vk_result_error {
	(
		pub enum $name: ident {
			vk {
				$(
					$( #[$attr: meta] )*
					$vk_error: ident
				),+
			}
			$( $other: tt )*
		}
	) => {
		use err_derive::Error;

		#[derive(Error, Debug)]
		pub enum $name {
			$(
				$( #[$attr] )*
				#[error(display = "{}", ash::vk::Result::$vk_error)]
				#[allow(non_camel_case_types)]
				$vk_error,
			)+

			$( $other )*
		}
		impl From<ash::vk::Result> for $name {
			fn from(err: ash::vk::Result) -> Self {
				match err {
					$(
						ash::vk::Result::$vk_error => $name::$vk_error,
					)+
					_ => unreachable!()
				}
			}
		}
	}
}

/// A utf8 encoded null-terminated string that is backed by a fixed-size array.
#[derive(Clone, Copy)]
pub struct VkSmallString {
	array: [c_char; Self::MAX_STRING_SIZE],
	len: usize
}
impl VkSmallString {
	// TODO: This should be the max of { MAX_PHYSICAL_DEVICE_NAME_SIZE, MAX_EXTENSION_NAME_SIZE, MAX_DESCRIPTION_SIZE, MAX_DRIVER_NAME_SIZE_KHR, MAX_DRIVER_INFO_SIZE_KHR }
	// Right now this seems to be the correct value
	pub const MAX_STRING_SIZE: usize = 256;

	pub unsafe fn from_c_string_unchecked(array: [c_char; Self::MAX_STRING_SIZE]) -> Self {
		VkSmallString {
			len: array.iter().enumerate().find(|(_, &byte)| byte == 0).unwrap().0,
			array
		}
	}
}
impl TryFrom<[c_char; Self::MAX_STRING_SIZE]> for VkSmallString {
	type Error = Utf8Error;

	fn try_from(array: [c_char; 256]) -> Result<Self, Self::Error> {
		let len = array.iter().enumerate().find(|(_, &byte)| byte == 0).unwrap().0;

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
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> { write!(f, "{}", self.deref()) }
}
impl Display for VkSmallString {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> { write!(f, "{}", self.deref()) }
}
