use ash::vk;
use ash::vk::SamplerAddressMode;


/// Sampler address mode containing only clamp modes.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(i32)]
pub enum AddressModeClamp {
	CLAMP_TO_EDGE = vk::SamplerAddressMode::CLAMP_TO_EDGE.as_raw(),
	CLAMP_TO_BORDER = vk::SamplerAddressMode::CLAMP_TO_BORDER.as_raw()
}
impl Into<vk::SamplerAddressMode> for AddressModeClamp {
	fn into(self) -> SamplerAddressMode {
		unsafe { std::mem::transmute(self) }
	}
}

unsafe_enum_variants! {
	#[derive(Debug, Copy, Clone)]
	enum SamplerCreateInfoInner {
		pub Unnormalized {
			filter: vk::Filter,
			address_mode: [AddressModeClamp; 2]
		} => {
			vk::SamplerCreateInfo::builder()
				.mag_filter(filter).min_filter(filter)
				.mipmap_mode(vk::SamplerMipmapMode::NEAREST)
				.address_mode_u(address_mode[0].into())
				.address_mode_v(address_mode[1].into())
				.anisotropy_enable(false)
				.min_lod(0.0).max_lod(0.0)
				.compare_enable(false)
				.unnormalized_coordinates(true)
			.build()
		},

		pub Subsampled {
			filter: vk::Filter,
			address_mode: [AddressModeClamp; 2]
		} => {
			vk::SamplerCreateInfo::builder()
				.flags(vk::SamplerCreateFlags::SUBSAMPLED_EXT)
				.mag_filter(filter).min_filter(filter)
				.mipmap_mode(vk::SamplerMipmapMode::NEAREST)
				.address_mode_u(address_mode[0].into())
				.address_mode_v(address_mode[1].into())
				.anisotropy_enable(false)
				.min_lod(0.0).max_lod(0.0)
				.compare_enable(false)
				.unnormalized_coordinates(false)
			.build()
		},

		pub Generic {
			min_filter: vk::Filter,
			mag_filter: vk::Filter,
			mipmap_mode: vk::SamplerMipmapMode,
			address_mode: [vk::SamplerAddressMode; 3],
			mip_lod_bias: f32,
			max_anisotropy: Option<f32>,
			compare_op: Option<vk::CompareOp>,
			min_lod: f32,
			max_lod: f32,
			border_color: Option<vk::BorderColor>
		} => {
			let mut builder = vk::SamplerCreateInfo::builder()
				.mag_filter(min_filter).min_filter(mag_filter)
				.mipmap_mode(mipmap_mode)
				.address_mode_u(address_mode[0].into())
				.address_mode_v(address_mode[1].into())
				.address_mode_w(address_mode[2].into())
				.mip_lod_bias(mip_lod_bias)
				.min_lod(min_lod).max_lod(max_lod.min(min_lod))
				.border_color(
					border_color.unwrap_or(vk::BorderColor::FLOAT_TRANSPARENT_BLACK)
				)
				.unnormalized_coordinates(false)
			;

			if let Some(max_anisotropy) = max_anisotropy {
				builder = builder.anisotropy_enable(true).max_anisotropy(max_anisotropy);
			}
			if let Some(compare_op) = compare_op {
				builder = builder.compare_enable(true).compare_op(compare_op);
			}

			builder.build()
		},

		{unsafe} pub Custom {
			info: vk::SamplerCreateInfo
		} => {
			info
		}
	} as pub SamplerCreateInfo impl Into<vk::SamplerCreateInfo>
	// TODO: the builder for this doesn't need a lifetime, but it has one
}