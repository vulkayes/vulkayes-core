use std::ffi::CStr;

use ash::vk;

use crate::pipeline::layout::PushConstantRange;

#[derive(Debug, Copy, Clone)]
pub enum ShaderEntryPoint<'a> {
	/// Most common "main" entry point name.
	Main,
	/// Custom entry point name.
	Custom(&'a CStr)
}
impl Default for ShaderEntryPoint<'static> {
	fn default() -> Self {
		ShaderEntryPoint::Main
	}
}
impl<'a> ShaderEntryPoint<'a> {
	pub fn to_cstr(self) -> &'a CStr {
		match self {
			ShaderEntryPoint::Main => unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") },
			ShaderEntryPoint::Custom(v) => v
		}
	}
}

/// Trait for values that are to be used as push constants.
pub unsafe trait PushConstantsTrait: Sized + std::fmt::Debug {
	const STAGE_FLAGS: vk::ShaderStageFlags =
		vk::ShaderStageFlags::from_raw(vk::ShaderStageFlags::VERTEX.as_raw() | vk::ShaderStageFlags::FRAGMENT.as_raw());
	const OFFSET_DIV_FOUR: u32 = 0;
	// TODO: Is there any way to force `Self` to **not** be a ZST?
	const SIZE_DIV_FOUR: u32 = (std::mem::size_of::<Self>() / 4) as u32;

	fn layout_range() -> PushConstantRange {
		PushConstantRange::new(
			Self::STAGE_FLAGS,
			Self::OFFSET_DIV_FOUR,
			std::num::NonZeroU32::new(Self::SIZE_DIV_FOUR).expect("Push constants block struct must have size of at least 4 bytes")
		)
	}

	// Returns self as an array of bytes.
	fn as_bytes(&self) -> &[u8] {
		unsafe {
			std::slice::from_raw_parts(
				self as *const Self as *const u8,
				Self::SIZE_DIV_FOUR as usize * 4
			)
		}
	}
}

/// Trait for values that can be used as specialization constants.
/// 
/// See `shader_specialization_constants` macro.
pub unsafe trait SpecializationConstantsTrait: std::fmt::Debug {
	fn specialization_map_entries() -> &'static [vk::SpecializationMapEntry];
	fn data(&self) -> &[u8];

	fn specialization_info<'a>(&'a self) -> vk::SpecializationInfoBuilder<'a> {
		vk::SpecializationInfo::builder()
			.map_entries(Self::specialization_map_entries())
			.data(self.data())
	}
}
unsafe impl SpecializationConstantsTrait for () {
	fn specialization_map_entries() -> &'static [vk::SpecializationMapEntry] { &[] }
	fn data(&self) -> &[u8] { &[] }

	fn specialization_info<'a>(&'a self) -> vk::SpecializationInfoBuilder<'a> {
		vk::SpecializationInfo::builder()
	}
}

#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct AlignedMatrix2<T: Copy + Default> {
	pub data: [[T; 4]; 2]
}
impl<T: Copy + Default> From<[[T; 3]; 2]> for AlignedMatrix2<T> {
	fn from(arr: [[T; 3]; 2]) -> Self {
		AlignedMatrix2 { data: [[arr[0][0], arr[0][1], arr[0][2], T::default()], [arr[1][0], arr[1][1], arr[0][1], T::default()]] }
	}
}

#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct AlignedMatrix3<T: Copy + Default> {
	pub data: [[T; 4]; 3]
}
impl<T: Copy + Default> From<[[T; 3]; 3]> for AlignedMatrix3<T> {
	fn from(arr: [[T; 3]; 3]) -> Self {
		AlignedMatrix3 {
			data: [
				[arr[0][0], arr[0][1], arr[0][2], T::default()],
				[arr[1][0], arr[1][1], arr[1][2], T::default()],
				[arr[2][0], arr[2][1], arr[2][2], T::default()]
			]
		}
	}
}

#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct AlignedMatrix4<T: Copy + Default> {
	pub data: [[T; 4]; 4]
}
impl<T: Copy + Default> From<[[T; 3]; 4]> for AlignedMatrix4<T> {
	fn from(arr: [[T; 3]; 4]) -> Self {
		AlignedMatrix4 {
			data: [
				[arr[0][0], arr[0][1], arr[0][2], T::default()],
				[arr[1][0], arr[1][1], arr[1][2], T::default()],
				[arr[2][0], arr[2][1], arr[2][2], T::default()],
				[arr[3][0], arr[3][1], arr[3][2], T::default()]
			]
		}
	}
}

/// Utility macro that can resolve GLSL shader types into Rust primitive types and Vulkan format values.
///
/// Usage:
/// ```
/// # use vulkayes_core::shader_util_macro;
/// # use vulkayes_core::ash::vk;
/// let shader_bool: shader_util_macro!(resolve_shader_type bool); // u32
/// let shader_mat4: shader_util_macro!(resolve_shader_type mat4); // [[f32; 4]; 4]
/// let shader_dmat2x3: shader_util_macro!(resolve_shader_type dmat2x3); // [[f64; 2]; 3]
///
/// let shader_bool_format: vk::Format = shader_util_macro!(resolve_shader_type_format bool); // R32_UINT
/// let shader_vec4_format: vk::Format = shader_util_macro!(resolve_shader_type_format vec4); // R32G32_SFLOAT
/// let shader_dvec2_format: vk::Format = shader_util_macro!(resolve_shader_type_format dvec3); // R64G64_SFLOAT
/// ```
#[macro_export]
macro_rules! shader_util_macro {
	// Types
	(resolve_shader_type bool) => {
		u32
	};
	(resolve_shader_type int) => {
		i32
	};
	(resolve_shader_type uint) => {
		u32
	};
	(resolve_shader_type float) => {
		f32
	};
	(resolve_shader_type double) => {
		f64
	};

	(resolve_shader_type bvec2) => {
		[u32; 2]
	};
	(resolve_shader_type bvec3) => {
		[u32; 3]
	};
	(resolve_shader_type bvec4) => {
		[u32; 4]
	};

	(resolve_shader_type ivec2) => {
		[i32; 2]
	};
	(resolve_shader_type ivec3) => {
		[i32; 3]
	};
	(resolve_shader_type ivec4) => {
		[i32; 4]
	};

	(resolve_shader_type uvec2) => {
		[u32; 2]
	};
	(resolve_shader_type uvec3) => {
		[u32; 3]
	};
	(resolve_shader_type uvec4) => {
		[u32; 4]
	};

	(resolve_shader_type vec2) => {
		[f32; 2]
	};
	(resolve_shader_type vec3) => {
		[f32; 3]
	};
	(resolve_shader_type vec4) => {
		[f32; 4]
	};

	(resolve_shader_type dvec2) => {
		[f64; 2]
	};
	(resolve_shader_type dvec3) => {
		[f64; 3]
	};
	(resolve_shader_type dvec4) => {
		[f64; 4]
	};

	// matrices
	(resolve_shader_type mat2) => {
		$crate::shader_util_macro!(resolve_shader_type mat2x2)
	};
	(resolve_shader_type mat2x2) => {
		[[f32; 2]; 2]
	};
	(resolve_shader_type mat2x3) => {
		$crate::shader::params::AlignedMatrix2<f32>
	};
	(resolve_shader_type mat2x4) => {
		[[f32; 4]; 2]
	};

	(resolve_shader_type mat3) => {
		$crate::shader_util_macro!(resolve_shader_type mat3x3)
	};
	(resolve_shader_type mat3x2) => {
		[[f32; 2]; 3]
	};
	(resolve_shader_type mat3x3) => {
		$crate::shader::params::AlignedMatrix3<f32>
	};
	(resolve_shader_type mat3x4) => {
		[[f32; 4]; 3]
	};

	(resolve_shader_type mat4) => {
		$crate::shader_util_macro!(resolve_shader_type mat4x4)
	};
	(resolve_shader_type mat4x2) => {
		[[f32; 2]; 4]
	};
	(resolve_shader_type mat4x3) => {
		$crate::shader::params::AlignedMatrix4<f32>
	};
	(resolve_shader_type mat4x4) => {
		[[f32; 4]; 4]
	};

	(resolve_shader_type dmat2) => {
		$crate::shader_util_macro!(resolve_shader_type dmat2x2)
	};
	(resolve_shader_type dmat2x2) => {
		[[f64; 2]; 2]
	};
	(resolve_shader_type dmat2x3) => {
		$crate::shader::params::AlignedMatrix2<f64>
	};
	(resolve_shader_type dmat2x4) => {
		[[f64; 4]; 2]
	};

	(resolve_shader_type dmat3) => {
		$crate::shader_util_macro!(resolve_shader_type dmat3x3)
	};
	(resolve_shader_type dmat3x2) => {
		[[f64; 2]; 3]
	};
	(resolve_shader_type dmat3x3) => {
		$crate::shader::params::AlignedMatrix3<f64>
	};
	(resolve_shader_type dmat3x4) => {
		[[f64; 4]; 3]
	};

	(resolve_shader_type dmat4) => {
		$crate::shader_util_macro!(resolve_shader_type dmat4x4)
	};
	(resolve_shader_type dmat4x2) => {
		[[f64; 2]; 4]
	};
	(resolve_shader_type dmat4x3) => {
		$crate::shader::params::AlignedMatrix4<f64>
	};
	(resolve_shader_type dmat4x4) => {
		[[f64; 4]; 4]
	};

	// Formats
	(resolve_shader_type_format bool) => {
		$crate::ash::vk::Format::R32_UINT
	};
	(resolve_shader_type_format int) => {
		$crate::ash::vk::Format::R32_SINT
	};
	(resolve_shader_type_format uint) => {
		$crate::ash::vk::Format::R32_UINT
	};
	(resolve_shader_type_format float) => {
		$crate::ash::vk::Format::R32_SFLOAT
	};
	(resolve_shader_type_format double) => {
		$crate::ash::vk::Format::R64_SFLOAT
	};

	(resolve_shader_type_format bvec2) => {
		$crate::ash::vk::Format::R32G32_UINT
	};
	(resolve_shader_type_format bvec3) => {
		$crate::ash::vk::Format::R32G32B32_UINT
	};
	(resolve_shader_type_format bvec4) => {
		$crate::ash::vk::Format::R32G32B32A32_UINT
	};

	(resolve_shader_type_format ivec2) => {
		$crate::ash::vk::Format::R32G32_SINT
	};
	(resolve_shader_type_format ivec3) => {
		$crate::ash::vk::Format::R32G32B32_SINT
	};
	(resolve_shader_type_format ivec4) => {
		$crate::ash::vk::Format::R32G32B32A32_SINT
	};

	(resolve_shader_type_format uvec2) => {
		$crate::ash::vk::Format::R32G32_UINT
	};
	(resolve_shader_type_format uvec3) => {
		$crate::ash::vk::Format::R32G32B32_UINT
	};
	(resolve_shader_type_format uvec4) => {
		$crate::ash::vk::Format::R32G32B32A32_UINT
	};

	(resolve_shader_type_format vec2) => {
		$crate::ash::vk::Format::R32G32_SFLOAT
	};
	(resolve_shader_type_format vec3) => {
		$crate::ash::vk::Format::R32G32B32_SFLOAT
	};
	(resolve_shader_type_format vec4) => {
		$crate::ash::vk::Format::R32G32B32A32_SFLOAT
	};

	(resolve_shader_type_format dvec2) => {
		$crate::ash::vk::Format::R64G64_SFLOAT
	};
	(resolve_shader_type_format dvec3) => {
		$crate::ash::vk::Format::R64G64B64_SFLOAT
	};
	(resolve_shader_type_format dvec4) => {
		$crate::ash::vk::Format::R64G64B64A64_SFLOAT
	};
}

#[macro_export]
macro_rules! shader_specialization_constants {
	(
		pub struct $name: ident {
			layout(
				$(local_size_x_id = $id_local_x: literal)?
				$(, local_size_y_id = $id_local_y: literal)?
				$(, local_size_z_id = $id_local_z: literal)?
				$(,)?
			) in;
			$(
				layout(constant_id = $id: literal) const $ty: ident $var: ident;
			)*
		}
	) => {
		$crate::shader_specialization_constants! {
			pub struct $name {
				$( layout(constant_id = $id_local_x) const uint local_size_x; )?
				$( layout(constant_id = $id_local_y) const uint local_size_y; )?
				$( layout(constant_id = $id_local_z) const uint local_size_z; )?
				$(
					layout(constant_id = $id) const $ty $var;
				)*
			}
		}
	};
	
	(
		pub struct $name: ident {
			$(
				layout(constant_id = $id: literal) const $ty: ident $var: ident;
			)+
		}
	) => {
		$crate::offsetable_struct! {
			#[derive(Copy, Clone)]
			pub struct $name {
				$(
					pub $var: $crate::shader_util_macro!(resolve_shader_type $ty)
				),+
			} repr(C) as Offsets // hidden by hygiene
		}
		impl $name {
			pub const SPECIALIZATION_MAP: &'static [$crate::ash::vk::SpecializationMapEntry] = &[
				$(
					$crate::ash::vk::SpecializationMapEntry {
						constant_id: $id,
						offset: $name::offsets().$var as u32,
						size: std::mem::size_of::<$crate::shader_util_macro!(resolve_shader_type $ty)>()
					}
				),+
			];
		}
		unsafe impl $crate::shader::params::SpecializationConstantsTrait for $name {
			fn specialization_map_entries() -> &'static [$crate::ash::vk::SpecializationMapEntry] {
				Self::SPECIALIZATION_MAP
			}

			fn data(&self) -> &[u8] {
				unsafe {
					std::slice::from_raw_parts(
						self as *const _ as *const u8,
						std::mem::size_of::<$name>()
					)
				}
			}
		}
		impl std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				f.debug_struct(stringify!($name))
					$(
						.field(
							concat!("layout(constant_id = ", stringify!($id), ") ", stringify!($var)),
							&self.$var
						)
					)+
					.finish()
			}
		}
	};
}

/// Generates input binding descriptions and input attribute descriptions for pipeline shaders.
///
/// Usage:
/// ```
/// # use vulkayes_core::ash::vk;
/// # use vulkayes_core::vertex_input_description;
/// vulkayes_core::offsetable_struct! {
/// 	struct Vertex {
/// 		position: [f32; 3],
/// 		color: [f32; 3]
/// 	} repr(C) as VertexOffsets
/// }
/// vulkayes_core::offsetable_struct! {
/// 	struct Normal {
/// 		Value: [f32; 3]
/// 	} repr(C) as NormalOffsets
/// }
/// let (bindings, attributes) = vertex_input_description!(
/// 	Vertex {@vk::VertexInputRate::VERTEX} {
/// 		 => layout(location = 0) in vec3 position; // Leaving out the field name defaults the offset to 0
/// 		.color => layout(location = 2) in vec3 color;
/// 	}
/// 	Normal {
/// 		=> layout(location = 1) in vec3 normal;
/// 	}
/// );
///
/// assert_eq!(bindings[0].binding, 0);
/// assert_eq!(bindings[0].stride, std::mem::size_of::<Vertex>() as u32);
/// assert_eq!(bindings[0].input_rate, vk::VertexInputRate::VERTEX);
///
/// assert_eq!(bindings[1].binding, 1);
/// assert_eq!(bindings[1].stride, std::mem::size_of::<Normal>() as u32);
/// assert_eq!(bindings[1].input_rate, vk::VertexInputRate::VERTEX);
///
/// assert_eq!(attributes[0].location, 0);
/// assert_eq!(attributes[0].binding, 0);
/// assert_eq!(attributes[0].format, vk::Format::R32G32B32_SFLOAT);
/// assert_eq!(attributes[0].offset, 0);
///
/// assert_eq!(attributes[1].location, 2);
/// assert_eq!(attributes[1].binding, 0);
/// assert_eq!(attributes[1].format, vk::Format::R32G32B32_SFLOAT);
/// assert_eq!(attributes[1].offset, 12);
///
/// assert_eq!(attributes[2].location, 1);
/// assert_eq!(attributes[2].binding, 1);
/// assert_eq!(attributes[2].format, vk::Format::R32G32B32_SFLOAT);
/// assert_eq!(attributes[2].offset, 0);
/// ```
#[macro_export]
macro_rules! vertex_input_description {
	(
		$(
			$struct_type: ty $({@ $rate: expr })? {
				$(
					$(.$struct_field: ident)? => layout(location = $location: expr) in $shader_type: ident $($name: ident)?;
				)+
			}
		)*
	) => {
		{
			let mut binding_number = 0;
			let input_bindings = [
				$(
					#[allow(unused_assignments)]
					{
						#[allow(unused_variables)]
						let input_rate = $crate::ash::vk::VertexInputRate::VERTEX;
						$(
							let input_rate = $rate;
						)?

						let desc = $crate::ash::vk::VertexInputBindingDescription {
							binding: binding_number,
							stride: std::mem::size_of::<$struct_type>() as u32,
							input_rate
						};

						binding_number += 1;

						desc
					}
				),*
			];

			let mut binding_number = 0;
			let input_attributes = [
				$(
					// This hack 2000 doesn't interfere with the multiple-item-expansion inner macro while allowing
					// `binding_number += 1` to be executed in the outer repetition only.
					if { binding_number += 1; false } { unsafe { std::hint::unreachable_unchecked() } } else
					$(
						{
							let location: u32 = $location;
							let input_type = $crate::shader_util_macro!(resolve_shader_type_format $shader_type);

							#[allow(unused_variables)]
							let offset: u32 = 0;
							$(
								let offset: u32 = <$struct_type>::offsets().$struct_field as u32;
							)?

							$crate::ash::vk::VertexInputAttributeDescription {
								location,
								binding: binding_number - 1,
								format: input_type,
								offset
							}
						},
					)+
				)*
			];

			// Ensure correct types in case of empty arrays
			let _: &[$crate::ash::vk::VertexInputBindingDescription] = &input_bindings;
			let _: &[$crate::ash::vk::VertexInputAttributeDescription] = &input_attributes;

			(input_bindings, input_attributes)
		}
	}
}

#[macro_export]
/// Creates a struct that is layout-compatible with glsl shader struct/block definition.
///
/// Usage:
/// ```
/// # use vulkayes_core::shader_block_struct;
/// shader_block_struct! {
/// 	#[derive(PartialEq)]
/// 	pub struct Foo {
/// 		vec4 color;
/// 		// glsl struct member definitions...
/// 	}
/// }
/// ```
///
/// Generates:
/// ```
/// #[repr(C)]
/// #[derive(Debug, Copy, Clone, Default)]
/// #[repr(align(4))]
/// #[derive(PartialEq)]
/// pub struct Foo {
/// 	pub color: [f32; 4]
/// }
/// ```
macro_rules! shader_block_struct {
	(
		$( #[$attribute: meta] )*
		$vv: vis struct $struct_name: ident {
			$(
				$ty: ident $name: ident;
			)+
		}
	) => {
		#[repr(C)]
		#[derive(Debug, Copy, Clone, Default)]
		#[repr(align(4))]
		$( #[$attribute] )*
		$vv struct $struct_name {
			$(
				pub $name: $crate::shader_util_macro!(resolve_shader_type $ty)
			),+
		}
	}
}

#[cfg(test)]
mod test {
	#[test]
	#[ignore]
	fn test_shader_params() {
		shader_specialization_constants! {
			pub struct VertexShaderSpecializationConstants {
				layout(constant_id = 0) const float foo;
				layout(constant_id = 1) const double bar;
				layout(constant_id = 2) const vec4 baz;
			}
		}

		eprintln!(
			"{:#?}",
			VertexShaderSpecializationConstants::offsets()
		);
		eprintln!(
			"{:#?}",
			VertexShaderSpecializationConstants::SPECIALIZATION_MAP
		);
	}
}
