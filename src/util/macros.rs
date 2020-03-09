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
		$(#[$attribute])*
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
				$v const $($safety)? fn $variant($( $( $variant_data: $variant_data ),+ )?) -> Self {
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
					_ => unreachable!("Cannot create {} from {}", stringify!($name), err) // TODO: Use unreachable unchecked in Release?
				}
			}
		}
		impl std::convert::TryInto<ash::vk::Result> for $name {
			type Error = $name;
			fn try_into(self) -> Result<ash::vk::Result, Self::Error> {
				#[allow(unreachable_patterns)]
				match self {
					$(
						$name::$vk_error => Ok(ash::vk::Result::$vk_error),
					)+
					_ => Err(self)
				}
			}
		}
	}
}

/// Implements `Deref`, `PartialEq`, `Eq` and `Hash` for a type based on its `Deref` implementation.
///
/// Since not all types deref directly into a handle, it is possible to provide a code fragment to get handle from deref target:
/// ```
/// impl_cmmon_handle_traits! {
/// 	impl [A: Debug] Deref, PartialEq, Eq, Hash for MyType [A] {
/// 		type Target = DerefTarget { field_on_self } // Derefs to `DerefTarget` by invoking `&self.field_on_self`
///
/// 		to_handle { .handle() } // Gets a handle from `DerefTarget` by invoking `self.field_on_self.handle()`
/// 	}
/// }
/// ```
///
/// this expands to
///
/// ```
/// impl<A: Debug> Deref for MyType<A> {
/// 	type Target = DerefTarget;
///
/// 	fn deref(&self) -> &Self::Target {
/// 		&self.field_on_self
/// 	}
/// }
/// impl<A: Debug> PartialEq for MyType<A> {
/// 	fn eq(&self, other: &Self) -> bool {
/// 		self.field_on_self.handle() == other.field_on_self.handle()
/// 	}
/// }
/// impl<A: Debug> Eq for MyType<A> { }
/// impl<A: Debug> Hash for MyType<A> {
/// 	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
/// 		self.field_on_self.handle().hash(state)
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! impl_common_handle_traits {
	(
		impl $([$($impl_gen: tt)*])? Deref, PartialEq, Eq, Hash for $tp: ty $([$($ty_gen: tt)*])? {
			type Target = $target: ty { $($target_code: tt)+ }

			$(
				to_handle { $($to_handle_code: tt)+ }
			)?
		}
	) => {
		impl $(<$($impl_gen)*>)? Deref for $tp $(<$($ty_gen)*>)? {
			type Target = $target;

			fn deref(&self) -> &Self::Target {
				&self.$($target_code)+
			}
		}
		impl $(<$($impl_gen)*>)? PartialEq for $tp $(<$($ty_gen)*>)? {
			fn eq(&self, other: &Self) -> bool {
				self.$($target_code)+ $( $($to_handle_code)+ )? == other.$($target_code)+ $( $($to_handle_code)+ )?
			}
		}
		impl $(<$($impl_gen)*>)? Eq for $tp $(<$($ty_gen)*>)? {}
		impl $(<$($impl_gen)*>)? std::hash::Hash for $tp $(<$($ty_gen)*>)? {
			fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
				self.$($target_code)+ $( $($to_handle_code)+ )? .hash(state)
			}
		}
	}
}