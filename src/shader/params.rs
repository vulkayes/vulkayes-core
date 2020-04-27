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
			$(
				layout(constant_id = $id: expr) const $ty: ident $var: ident;
			)+
		}
	) => {
		#[repr(C)]
			$crate::offsetable_struct! {
				#[derive(Debug, Copy, Clone)]
				pub struct $name {
					$(
						pub $var: $crate::shader_util_macro!(resolve_shader_type $ty)
					),+
				} repr(C) as Offsets // hidden by hygiene
			}
			impl $name {
				pub const fn specialization_map_entries() -> &'static [$crate::ash::vk::SpecializationMapEntry] {
					const ENTRIES: &'static [$crate::ash::vk::SpecializationMapEntry] = &[
						$(
							$crate::ash::vk::SpecializationMapEntry {
								constant_id: $id,
								offset: $name::offsets().$var as u32,
								size: std::mem::size_of::<$crate::shader_util_macro!(resolve_shader_type $ty)>()
							}
						),+
					];

					ENTRIES
				}

				pub fn specialization_info(&self) -> $crate::ash::vk::SpecializationInfoBuilder {
					$crate::ash::vk::SpecializationInfo::builder()
						.map_entries(Self::specialization_map_entries())
						.data(
							unsafe {
								std::slice::from_raw_parts(
									self as *const _ as *const u8,
									std::mem::size_of::<$name>()
								)
							}
						)
				}
			}
	}
}

#[macro_export]
macro_rules! vertex_input_attributes {
	(
		$(
			layout(location = $location: expr $(, binding = $binding: expr)? $(, offset = $offset: expr)?) in $ty: ident $($name: ident)?;
		)+
	) => {
		[
			$(
				{
					let location: u32 = $location;
					let input_type = $crate::shader_util_macro!(resolve_shader_type_format $ty);

					#[allow(unused_variables)]
					let binding: u32 = 0;
					$(
						let binding: u32 = $binding;
					)?

					#[allow(unused_variables)]
					let offset: u32 = 0;
					$(
						let offset: u32 = $offset;
					)?

					$crate::ash::vk::VertexInputAttributeDescription {
						location,
						binding,
						format: input_type,
						offset
					}
				}
			),+
		]
	}
}

#[cfg(test)]
mod test {
	#[test]
	fn test_shader_params() {
		shader_specialization_constants! {
			pub struct VertexShaderSpecializationConstants {
				layout(constant_id = 0) const float foo;
				layout(constant_id = 1) const double bar;
				layout(constant_id = 2) const vec4 baz;
			}
		}

		let attributes = vertex_input_attributes! {
			layout(location = 0, binding = 0) in vec3;
			layout(location = 1) in vec4;
			layout(location = 1, binding = 0, offset = 12) in double;
		};

		eprintln!("{:#?}", VertexShaderSpecializationConstants::offsets());
		eprintln!(
			"{:#?}",
			VertexShaderSpecializationConstants::specialization_map_entries()
		);

		eprintln!("{:#?}", attributes);
	}
}
