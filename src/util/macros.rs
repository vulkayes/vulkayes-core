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

/// Generates a public enum that derives `thiserror::Error` with `VkResult` variants and their `From` impls.
#[macro_export]
macro_rules! vk_result_error {
	(
		$( #[$attribute: meta] )*
		pub enum $name: ident {
			vk {
				$(
					$( #[$variant_attribute: meta] )*
					$vk_error: ident
				),+
			}
			$( $other: tt )*
		}
	) => {
		#[allow(unused_imports)]
		use thiserror::*;

		$( #[$attribute] )*
		#[derive(Error)]
		pub enum $name {
			$(
				$( #[$variant_attribute] )*
				#[error("{}", ash::vk::Result::$vk_error)]
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
