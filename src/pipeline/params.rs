use ash::vk;

unsafe impl crate::util::transparent::Transparent for vk::PipelineShaderStageCreateInfoBuilder<'_> {
	type Target = vk::PipelineShaderStageCreateInfo;
}
unsafe impl crate::util::transparent::Transparent
	for vk::PipelineColorBlendAttachmentStateBuilder<'_>
{
	type Target = vk::PipelineColorBlendAttachmentState;
}

unsafe_enum_variants! {
	#[derive(Debug, Copy, Clone)]
	enum PolygonModeInner {
		pub Point => {
			(vk::PolygonMode::POINT, vk::CullModeFlags::NONE, vk::FrontFace::COUNTER_CLOCKWISE, 1.0)
		},
		pub Line { width: f32 } => {
			(vk::PolygonMode::LINE, vk::CullModeFlags::NONE, vk::FrontFace::COUNTER_CLOCKWISE, width)
		},
		pub LineDynamic => {
			(vk::PolygonMode::LINE, vk::CullModeFlags::NONE, vk::FrontFace::COUNTER_CLOCKWISE, f32::NAN)
		},
		pub Fill {
			cull_mode: vk::CullModeFlags,
			front_face: vk::FrontFace
		} => {
			(vk::PolygonMode::FILL, cull_mode, front_face, 1.0)
		},

		{unsafe} pub Custom {
			polygon_mode: vk::PolygonMode,
			cull_mode: vk::CullModeFlags,
			front_face: vk::FrontFace,
			line_width: f32
		} => {
			(polygon_mode, cull_mode, front_face, line_width)
		}
	} as pub PolygonMode impl Into<(vk::PolygonMode, vk::CullModeFlags, vk::FrontFace, f32)>
}
impl Default for PolygonMode {
	fn default() -> Self {
		PolygonMode::Fill(vk::CullModeFlags::NONE, vk::FrontFace::COUNTER_CLOCKWISE)
	}
}

#[derive(Debug, Copy, Clone)]
pub enum DepthBias {
	Disabled,
	Enabled {
		constant_factor: f32,
		clamp: f32,
		slope_factor: f32
	},
	Dynamic
}
impl Default for DepthBias {
	fn default() -> Self {
		DepthBias::Disabled
	}
}
impl Into<(bool, f32, f32, f32)> for DepthBias {
	fn into(self) -> (bool, f32, f32, f32) {
		match self {
			DepthBias::Disabled => (false, 0.0, 0.0, 0.0),
			DepthBias::Enabled {
				constant_factor,
				clamp,
				slope_factor
			} => (true, constant_factor, clamp, slope_factor),
			DepthBias::Dynamic => (true, f32::NAN, f32::NAN, f32::NAN)
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub enum SampleShading {
	Disabled,
	Enabled { min_sample_shading: f32 }
}
impl Default for SampleShading {
	fn default() -> Self {
		SampleShading::Disabled
	}
}
impl Into<(bool, f32)> for SampleShading {
	fn into(self) -> (bool, f32) {
		match self {
			SampleShading::Disabled => (false, 1.0),
			SampleShading::Enabled { min_sample_shading } => (true, min_sample_shading)
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub enum DepthTest {
	Disabled,
	Enabled(vk::CompareOp),
	EnabledReadonly(vk::CompareOp)
}
impl Default for DepthTest {
	fn default() -> Self {
		DepthTest::Enabled(vk::CompareOp::LESS)
	}
}
impl Into<(bool, bool, vk::CompareOp)> for DepthTest {
	fn into(self) -> (bool, bool, vk::CompareOp) {
		match self {
			DepthTest::Disabled => (false, false, vk::CompareOp::NEVER),
			DepthTest::Enabled(op) => (true, true, op),
			DepthTest::EnabledReadonly(op) => (true, false, op)
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub enum DepthBoundsTest {
	Disabled,
	Enabled(f32, f32),
	Dynamic
}
impl Default for DepthBoundsTest {
	fn default() -> DepthBoundsTest {
		DepthBoundsTest::Disabled
	}
}
impl Into<(bool, f32, f32)> for DepthBoundsTest {
	fn into(self) -> (bool, f32, f32) {
		match self {
			DepthBoundsTest::Disabled => (false, 0.0, 0.0),
			DepthBoundsTest::Enabled(min, max) => (true, min, max),
			DepthBoundsTest::Dynamic => (true, f32::NAN, f32::NAN)
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub enum StencilTest {
	Disabled,
	Enabled {
		fail_op: [vk::StencilOp; 2],
		pass_op: [vk::StencilOp; 2],
		depth_fail_op: [vk::StencilOp; 2],
		compare_op: [vk::CompareOp; 2],
		compare_mask: Option<[u32; 2]>,
		write_mask: Option<[u32; 2]>,
		reference: Option<[u32; 2]>
	}
}
impl Default for StencilTest {
	fn default() -> StencilTest {
		StencilTest::Disabled
	}
}
impl
	Into<(
		bool,
		vk::StencilOpState,
		vk::StencilOpState,
		bool,
		bool,
		bool
	)> for StencilTest
{
	fn into(
		self
	) -> (
		bool,
		vk::StencilOpState,
		vk::StencilOpState,
		bool,
		bool,
		bool
	) {
		match self {
			StencilTest::Disabled => (
				false,
				Default::default(),
				Default::default(),
				false,
				false,
				false
			),
			StencilTest::Enabled {
				fail_op,
				pass_op,
				depth_fail_op,
				compare_op,
				compare_mask,
				write_mask,
				reference
			} => {
				let cmp_mask = compare_mask.unwrap_or([0, 0]);
				let wrt_mask = write_mask.unwrap_or([0, 0]);
				let rfc = reference.unwrap_or([0, 0]);

				(
					true,
					vk::StencilOpState {
						fail_op: fail_op[0],
						pass_op: pass_op[0],
						depth_fail_op: depth_fail_op[0],
						compare_op: compare_op[0],
						compare_mask: cmp_mask[0],
						write_mask: wrt_mask[0],
						reference: rfc[0]
					},
					vk::StencilOpState {
						fail_op: fail_op[1],
						pass_op: pass_op[1],
						depth_fail_op: depth_fail_op[1],
						compare_op: compare_op[1],
						compare_mask: cmp_mask[1],
						write_mask: wrt_mask[1],
						reference: rfc[1]
					},
					compare_mask.is_none(),
					write_mask.is_none(),
					reference.is_none()
				)
			}
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub enum BlendLogicOp {
	Disabled,
	Enabled(vk::LogicOp)
}
impl Default for BlendLogicOp {
	fn default() -> BlendLogicOp {
		BlendLogicOp::Disabled
	}
}
impl Into<(bool, vk::LogicOp)> for BlendLogicOp {
	fn into(self) -> (bool, vk::LogicOp) {
		match self {
			BlendLogicOp::Disabled => (false, Default::default()),
			BlendLogicOp::Enabled(op) => (true, op)
		}
	}
}

/// Expands to a tuple of `(vk::Viewport, vk::Rect2D)` or into a tuple of `([vk::Viewport], [vk::Rect2D], bool, bool)`.
///
/// Syntax: `area offset? depth? scissor?`
/// * `area` - expression: `[$w, $h]` specifying width and height of the viewport (in `f32`)
/// * `offset?` - expression: `+ [$x, $y]` specifying offset of viewport, default: `[0.0f32, 0.0f32]`
/// * `depth?` - expression: `: [$min, $max]` specifying minimum and maximum depth, default: `[0.0f32, 1.0f32]`
/// * `scissor?` - expression: `@ [$x, $y, $width, $height]` specifying scissor region, default: `[0i32, 0i32, i32::MAX as u32, i32::MAX as u32]`
///
/// Example:
/// ```
/// # use vulkayes_core::viewport_scissor_expr;
/// viewport_scissor_expr!(
/// 	[100.0, 200.0] + [10.0, 20.0] : [0.0, 1.0] @ [0, 0, 100, 200]
/// );
/// ```
#[macro_export]
macro_rules! viewport_scissor_expr {
	(
		[
			$(
				dynamic @ [$sc_left: expr, $sc_top: expr, $sc_width: expr, $sc_height: expr]
			),+ $(,)?
		]
	) => {
		{
			let viewports = [
				$(
					{
						let _ = $sc_left; // hack for macro
						$crate::ash::vk::Viewport::default()
					}
				),+
			];

			let scissors = [
				$(
					$crate::viewport_scissor_expr!([0.0, 0.0] @ [$sc_left, $sc_top, $sc_width, $sc_height]).1
				),+
			];

			(viewports, scissors, true, false)
		}
	};

	(
		[
			$(
				[$width: expr, $height: expr] $(+ [$left: expr, $top: expr])? $(: [$near: expr, $far: expr])? @ dynamic
			),+ $(,)?
		]
	) => {
		{
			let viewports = [
				$(
					$crate::viewport_scissor_expr!(
						[$width, $height] $(+ [$left, $top])? $(: [$near, $far])?
					).0
				),+
			];

			let scissors = [
				$(
					{
						let _ = $width; // hack for macro
						$crate::ash::vk::Rect2D::default()
					}
				),+
			];

			(viewports, scissors, false, true)
		}
	};

	(
		dynamic[$count: expr]
	) => {
		{
			let viewports = [$crate::ash::vk::Viewport::default(); $count];
			let scissors = [$crate::ash::vk::Rect2D::default(); $count];

			(viewports, scissors, true, true)
		}
	};

	(
		[
			$(
				[$width: expr, $height: expr]
					$(+ [$left: expr, $top: expr])?
					$(: [$near: expr, $far: expr])?
					$(@ [$sc_left: expr, $sc_top: expr, $sc_width: expr, $sc_height: expr])?
			),* $(,)?
		]
	) => {
		{
			let viewports = [
				$(
					$crate::viewport_scissor_expr!(
						[$width, $height] $(+ [$left, $top])? $(: [$near, $far])?
					).0
				),*
			];

			let scissors = [
				$(
					$crate::viewport_scissor_expr!(
						[$width, $height] $(+ [$left, $top])? $(: [$near, $far])? $(@ [$sc_left, $sc_top, $sc_width, $sc_height])?
					).1
				),*
			];

			(viewports, scissors, false, false)
		}
	};

	(
		[$width: expr, $height: expr]
			$(+ [$left: expr, $top: expr])?
			$(: [$near: expr, $far: expr])?
			$(@ [$sc_left: expr, $sc_top: expr, $sc_width: expr, $sc_height: expr])?
	) => {
		{
			let viewport = {
				#[allow(unused_assignments, unused_mut)]
				let mut x: f32 = 0.0;
				#[allow(unused_assignments, unused_mut)]
				let mut y: f32 = 0.0;
				$(
					x = $left;
					y = $top;
				)?

				#[allow(unused_assignments)]
				let width: f32 = $width;
				#[allow(unused_assignments)]
				let height: f32 = $height;

				#[allow(unused_assignments, unused_mut)]
				let mut min_depth: f32 = 0.0;
				#[allow(unused_assignments, unused_mut)]
				let mut max_depth: f32 = 1.0;
				$(
					min_depth = $near;
					max_depth = $far;
				)?

				$crate::ash::vk::Viewport {
					x,
					y,
					width,
					height,
					min_depth,
					max_depth
				}
			};

			let scissor = {
				#[allow(unused_assignments, unused_mut)]
				let mut x: i32 = 0;
				#[allow(unused_assignments, unused_mut)]
				let mut y: i32 = 0;

				#[allow(unused_assignments, unused_mut)]
				let mut width: u32 = i32::MAX as u32;
				#[allow(unused_assignments, unused_mut)]
				let mut height: u32 = i32::MAX as u32;

				$(
					x = $sc_left;
					y = $sc_top;
					width = $sc_width;
					height = $sc_height;
				)?

				$crate::ash::vk::Rect2D {
					offset: $crate::ash::vk::Offset2D {
						x,
						y
					},
					extent: $crate::ash::vk::Extent2D {
						width,
						height
					}
				}
			};

			(viewport, scissor)
		}
	};
}

/// Expands to `vk::PipelineColorBlendAttachmentStateBuilder`.
///
/// Syntax: `color : alpha & mask`
/// * `color` - expression: `($src_factor) * src {$op} ($dst_factor) * dst` specifying the source and destination factors (`vk::BlendFactor`) and blending operation (`vk::BlendOp`).
/// * `alpha` - expression: `($src_factor) * src {$op} ($dst_factor) * dst` specifying the source and destination factors (`vk::BlendFactor`) and blending operation (`vk::BlendOp`).
/// * `mask` - value of type `vk::ColorComponentFlags`
///
/// The macro also accepts the token `disabled & mask`, which returns a builder with blending disabled.
///
/// Example:
/// ```
/// # use vulkayes_core::color_blend_state_expr;
/// # use vulkayes_core::ash::vk;
/// // Blends based on alpha, stores the new alpha, doesn't mask anything
/// color_blend_state_expr!(
/// 	(S * SRC_ALPHA) ADD (D * ONE_MINUS_SRC_ALPHA)
/// 		: (S * ONE) SUBTRACT (D * ZERO)
/// 		& vk::ColorComponentFlags::all()
/// );
/// // Disables blending, doesn't mask anything
/// color_blend_state_expr!(
/// 	disabled & vk::ColorComponentFlags::all()
/// );
/// // Same as the first one, but variables are expressions instead of identifiers.
/// color_blend_state_expr!(
/// 	(S * vk::BlendFactor::SRC_ALPHA) {vk::BlendOp::ADD} (D * vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
/// 		: (S * vk::BlendFactor::ONE) {vk::BlendOp::SUBTRACT} (D * vk::BlendFactor::ZERO)
/// 		& vk::ColorComponentFlags::all()
/// );
/// ```
#[macro_export]
macro_rules! color_blend_state_expr {
	(
		disabled & $color_write_mask: expr
	) => {
		$crate::ash::vk::PipelineColorBlendAttachmentState::builder().blend_enable(false)
			.color_write_mask($color_write_mask)
	};

	(
		(S * $src_color_blend_factor: ident) $color_blend_op: ident (D * $dst_color_blend_factor: ident)
			: (S * $src_alpha_blend_factor: ident) $alpha_blend_op: ident (D * $dst_alpha_blend_factor: ident)
			& $color_write_mask: expr
	) => {
		{
			use $crate::ash::vk::{BlendFactor, BlendOp};

			$crate::color_blend_state_expr!(
				(S * BlendFactor::$src_color_blend_factor) {BlendOp::$color_blend_op} (D * BlendFactor::$dst_color_blend_factor)
					: (S * BlendFactor::$src_alpha_blend_factor) {BlendOp::$color_blend_op} (D * BlendFactor::$dst_alpha_blend_factor)
					& $color_write_mask
			)
		}
	};

	(
		(S * $src_color_blend_factor: expr) {$color_blend_op: expr} (D * $dst_color_blend_factor: expr)
			: (S * $src_alpha_blend_factor: expr) {$alpha_blend_op: expr} (D * $dst_alpha_blend_factor: expr)
			& $color_write_mask: expr
	) => {
		$crate::ash::vk::PipelineColorBlendAttachmentState::builder().blend_enable(true)
			.src_color_blend_factor($src_color_blend_factor)
			.dst_color_blend_factor($dst_color_blend_factor)
			.color_blend_op($color_blend_op)
			.src_alpha_blend_factor($src_alpha_blend_factor)
			.dst_alpha_blend_factor($dst_alpha_blend_factor)
			.alpha_blend_op($alpha_blend_op)
			.color_write_mask($color_write_mask)
	};
}

/// Graphics pipeline creation macro that makes it easier to specify parameters.
///
/// This macro expands to an item. To use the generated `vk::GraphicsPipelineCreateInfoBuilder`,
/// the variable expanded from `$create_info_variable_name` is introduced into the scope after calling this macro.
///
/// The builder is only valid for the lifetime of the calling scope because it borrows from variables declared inside
/// this macro. Calling `build` on the builder and then returning it is undefined behavior!. TODO
///
/// The parameters are split into several sections grouping similar paramters. The following list shows
/// separate sections, their fields and the types of arguments expected. `?` marks an optional parameter.
///
/// * **Shaders** - Parameters affecting shader stages, input descriptions and primitive topology.
/// 	* `stages` - array of binding expressions: `module, entry?, spec? => point`:
/// 		* `module` - value of type [`ShaderModule`](shader/struct.ShaderModule.html)
/// 		* `entry?` - value of type [`ShaderEntryPoint`](shader/params/enum.ShaderEntryPoint.html), default: `ShaderEntryPoint::default()`
/// 		* `spec?` - any value defining `fn specialization_info(&self) -> vk::SpecializationInfoBuilder`, default: unset (null pointer)
/// 		* `point` -> value of type `vk::ShaderStageFlags`
/// 	* `input` - tokens passed directly to [`vertex_input_description!`](macro.vertex_input_description.html) macro
/// 	* `topology` - value of type `vk::PrimitiveTopology`
/// 	* `primitive_restart?` - value of type `bool`, default: `false`
/// * **Tessellation** - Parameters affecting tessellation.
/// 	* `patch_control_points`? - value of type `u32`, default: `0`
/// * **Viewport** - Parameters affecting viewports and scissors.
/// 	* `viewports` - tokens passed directly to [`viewport_scissor_expr!`](macro.viewport_scissor_expr.html) macro
/// * **Rasterization** - Parameters affecting rasterization, clipping and clamping.
/// 	* `depth_clamp?` - value of type `bool`, default: `false`
/// 	* `depth_clip?` - value of type `bool`, default: unset (extension struct not passed)
/// 	* `discard?` - value of type `bool`, default: `false`
/// 	* `polygon_mode` - value of type [`PolygonMode`](pipeline/params/struct.PolygonMode.html)
/// 	* `depth_bias?` - value of type [`DepthBias`](pipeline/params/enum.DepthBias.html), default: `DepthBias::default()`
/// * **Multisampling** - Parameters affecting multisampling.
/// 	* `samples` - value of type `vk::SampleCountFlags`
/// 	* `sample_shading?` - value of type [`SampleShading`](pipeline/params/enum.SampleShading.html), default: `SampleShading::default()`
/// 	* `sample_mask?` - value that borrows as `&[u32]`, default: unset (null pointer)
/// 	* `alpha_to_coverage?` - value of type `bool`, default: `false`
/// 	* `alpha_to_one?` - value of type `bool`, default: `false`
/// * **DepthStencil** - Parameters affecting depth and stencil tests.
/// 	* `depth` - value of type [`DepthTest`](pipeline/params/enum.DepthTest.html)
/// 	* `depth_bounds`? - value of type [`DepthBoundsTest`](pipeline/params/enum.DepthBoundsTest.html), default: `DepthBoundsTest::default()`
/// 	* `stencil`? - value of type [`StencilTest`](pipeline/params/enum.StencilTest.html), default: `StencilTest::default()`
/// * **ColorBlend** - Parameters affecting color blending and operations.
/// 	* `logic_op?` - value of type [`BlendLogicOp`](pipeline/params/enum.BlendLogicOp.html), default: `BlendLogicOp::default()`
/// 	* `attachments` -  array of blending expressions passed directly to [`color_blend_state_expr!`](macro.color_blend_state_expr.html) macro
/// 	* `blend_constants?` - value of type `Option<[f32; 4]>`, default: `Some([0.0; 4])`, `None` means to enable dynamic state
/// * **Deps** - Parameters needed as dependencies.
/// 	* `layout` - any value defining `fn handle(&self) -> vk::PipelineLayout`
/// 	* `render_pass` - any value defining `fn handle(&self) -> vk::RenderPass`
/// 	* `subpass?` - value of type `u32`, default: `0`
///
/// Note that some sections are optional altogether, however, not specifying a section means it won't be included at all in the
/// create info and no defaults will be provided (the struct pointer will be null). Commonly only the `Tessellation` section is left out,
/// but with rasterization disabled the `Viewport`, `Multisampling`, `DepthStencil` and `ColorBlend` sections can be left out as well.
/// `DepthStencil` and `ColorBlend` also have additional cases where they can be left out: <https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkGraphicsPipelineCreateInfo.html>.
#[macro_export]
macro_rules! create_graphics_pipeline {
	(
		@Shaders($output_builder: expr)
		stages: [
			$(
				$stage: expr $(, $entry_name: expr $(, $specialization: expr)?)? => $stage_type: expr
			),* $(,)?
		] $(,)?
		input: {
			$($input_tt: tt)*
		} $(,)?
		topology: $topology: expr
		$(, primitive_restart: $primitive_restart: expr)?
		$(,)?
	) => {
		let specialization_infos = [
			$(
				{
					let mut info = None::<$crate::ash::vk::SpecializationInfoBuilder>;
					$(
						$(
							info = Some($specialization.specialization_info());
						)?
					)?

					info
				}
			),*
		];
		let _: &[Option<$crate::ash::vk::SpecializationInfoBuilder>] = &specialization_infos;

		#[allow(unused_variables, unused_mut)]
		let mut counter: usize = 0;
		let stages = [
			$(
				{
					let mut entry_name: $crate::shader::params::ShaderEntryPoint = Default::default();
					$(
						entry_name = $entry_name;
					)?

					counter += 1;
					$stage.stage_create_info(
						$stage_type,
						entry_name,
						specialization_infos[counter - 1].as_ref()
					)
				}
			),*
		];
		let _: &[$crate::ash::vk::PipelineShaderStageCreateInfoBuilder] = &stages;

		let (shader_input_bindings, shader_input_attributes) = $crate::vertex_input_description!(
			$($input_tt)*
		);
		let input_assembly = $crate::ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
			.topology($topology)
			.primitive_restart_enable(
				{
					#[allow(unused_mut)]
					let mut restart = false;
					$(
						restart = $primitive_restart;
					)?

					restart
				}
			)
		;

		let input_state = $crate::ash::vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_binding_descriptions(&shader_input_bindings)
			.vertex_attribute_descriptions(&shader_input_attributes)
		;

		$output_builder = $output_builder
			.stages(
				$crate::util::transparent::Transparent::transmute_slice(&stages)
			)
			.vertex_input_state(&input_state)
			.input_assembly_state(&input_assembly)
		;
	};

	(
		@Tessellation($output_builder: expr)
		patch_control_points: $patch_control_points: expr
		$(, domain_origin: $domain_origin: expr)?
	) => {
		#[allow(unused_mut)]
		let mut builder = $crate::ash::vk::PipelineTessellationStateCreateInfo::builder()
			.patch_control_points($patch_control_points)
		;
		$(
			let mut domain_origin_info = $crate::ash::vk::PipelineTessellationDomainOriginStateCreateInfo::builder()
				.domain_origin($domain_origin)
			;
			builder = builder.push_next(&mut domain_origin_info);
		)?

		$output_builder = $output_builder
			.tessellation_state(&builder)
		;
	};

	(
		@Viewport($output_builder: expr, $dynamic_info: expr)
		viewports: {
			$($viewports_tt: tt)+
		} $(,)?
	) => {
		let (viewports, scissors, dynamic_viewport, dynamic_scissors) = $crate::viewport_scissor_expr!(
			$($viewports_tt)+
		);

		let builder = $crate::ash::vk::PipelineViewportStateCreateInfo::builder()
			.viewports(&viewports)
			.scissors(&scissors)
		;

		$output_builder = $output_builder.viewport_state(&builder);

		if dynamic_viewport {
			$dynamic_info.push($crate::ash::vk::DynamicState::VIEWPORT);
		}
		if dynamic_scissors {
			$dynamic_info.push($crate::ash::vk::DynamicState::SCISSOR);
		}
	};

	(
		@Rasterization($output_builder: expr, $dynamic_info: expr)
		$(depth_clamp: $depth_clamp: expr,)?
		$(depth_clip: $depth_clip: expr,)?
		$(discard: $discard: expr,)?
		polygon_mode: $polygon_mode: expr
		$(, depth_bias: $depth_bias: expr)?
		$(,)?
	) => {
		#[allow(unused_assignments, unused_mut)]
		let mut builder = $crate::ash::vk::PipelineRasterizationStateCreateInfo::builder();
		$(
			let mut depth_clip = $crate::ash::vk::PipelineRasterizationDepthClipStateCreateInfoEXT::builder().depth_clip_enable($depth_clip);
			builder = builder.push_next(&mut depth_clip);
		)?

		#[allow(unused_assignments, unused_mut)]
		let mut depth_clamp = false;
		$(
			depth_clamp = $depth_clamp;
		)?
		#[allow(unused_assignments, unused_mut)]
		let mut discard = false;
		$(
			discard = $discard;
		)?

		let polygon_info: $crate::pipeline::params::PolygonMode = $polygon_mode;
		let (polygon_mode, cull_mode, front_face, line_width) = polygon_info.into();

		#[allow(unused_assignments, unused_mut)]
		let mut depth_bias: $crate::pipeline::params::DepthBias = Default::default();
		$(
			depth_bias = $depth_bias;
		)?
		let (depth_bias_enable, depth_bias_constant_factor, depth_bias_clamp, depth_bias_slope_factor) = depth_bias.into();

		builder = builder
			.depth_clamp_enable(depth_clamp)
			.rasterizer_discard_enable(discard)
			.polygon_mode(polygon_mode)
			.cull_mode(cull_mode)
			.front_face(front_face)
			.depth_bias_enable(depth_bias_enable)
			.depth_bias_constant_factor(depth_bias_constant_factor)
			.depth_bias_clamp(depth_bias_clamp)
			.depth_bias_slope_factor(depth_bias_slope_factor)
			.line_width(line_width)
		;

		$output_builder = $output_builder.rasterization_state(&builder);

		if line_width.is_nan() {
			$dynamic_info.push($crate::ash::vk::DynamicState::LINE_WIDTH);
		}
		if depth_bias_enable && (depth_bias_constant_factor.is_nan() || depth_bias_clamp.is_nan() || depth_bias_slope_factor.is_nan()) {
			$dynamic_info.push($crate::ash::vk::DynamicState::DEPTH_BIAS);
		}
	};

	(
		@Multisampling($output_builder: expr)
		samples: $samples: expr
		$(, sample_shading: $sample_shading: expr)?
		$(, sample_mask: $sample_mask: expr)?
		$(, alpha_to_coverage: $alpha_to_coverage: expr)?
		$(, alpha_to_one: $alpha_to_one: expr)?
		$(,)?
	) => {
		#[allow(unused_assignments, unused_mut)]
		let mut sample_shading = $crate::pipeline::params::SampleShading::default();
		$(
			sample_shading = $sample_shading;
		)?
		let (sample_shading_enable, min_sample_shading) = sample_shading.into();

		#[allow(unused_assignments, unused_mut)]
		let mut alpha_to_coverage = false;
		$(
			alpha_to_coverage = $alpha_to_coverage;
		)?
		#[allow(unused_assignments, unused_mut)]
		let mut alpha_to_one = false;
		$(
			alpha_to_one = $alpha_to_one;
		)?

		#[allow(unused_assignments, unused_mut)]
		let mut builder = $crate::ash::vk::PipelineMultisampleStateCreateInfo::builder()
			.rasterization_samples($samples)
			.sample_shading_enable(sample_shading_enable)
			.min_sample_shading(min_sample_shading)
			.alpha_to_coverage_enable(alpha_to_coverage)
			.alpha_to_one_enable(alpha_to_one)
		;

		$(
			let sample_mask_array = $sample_mask;
			let sample_mask_array_slice: &[u32] = $sample_mask_array;
			builder = builder.sample_mask(sample_mask_array_slice);
		)?

		$output_builder = $output_builder.multisample_state(&builder);
	};

	(
		@DepthStencil($output_builder: expr, $dynamic_info: expr)
		depth: $depth_test: expr
		$(, depth_bounds: $depth_bounds_test: expr)?
		$(, stencil: $stencil_test: expr)?
		$(,)?
	) => {
		let depth_test: $crate::pipeline::params::DepthTest = $depth_test;
		let (depth_test_enable, depth_write_enable, depth_compare_op) = depth_test.into();

		#[allow(unused_assignments, unused_mut)]
		let mut depth_bounds_test = $crate::pipeline::params::DepthBoundsTest::default();
		$(
			depth_bounds_test = $depth_bounds_test;
		)?
		let (depth_bounds_test_enable, min_depth_bounds, max_depth_bounds) = depth_bounds_test.into();

		#[allow(unused_assignments, unused_mut)]
		let mut stencil_test = $crate::pipeline::params::StencilTest::default();
		$(
			stencil_test = $stencil_test;
		)?
		let (stencil_test_enable, front, back, stencil_comare_dynamic, stencil_write_dynamic, stencil_reference_dynamic) = stencil_test.into();

		let builder = $crate::ash::vk::PipelineDepthStencilStateCreateInfo::builder()
			.depth_test_enable(depth_test_enable)
			.depth_write_enable(depth_write_enable)
			.depth_compare_op(depth_compare_op)
			.depth_bounds_test_enable(depth_bounds_test_enable)
			.stencil_test_enable(stencil_test_enable)
			.front(front)
			.back(back)
			.min_depth_bounds(min_depth_bounds)
			.max_depth_bounds(max_depth_bounds)
		;

		$output_builder = $output_builder
			.depth_stencil_state(&builder)
		;

		if depth_bounds_test_enable && (min_depth_bounds.is_nan() || max_depth_bounds.is_nan()) {
			$dynamic_info.push($crate::ash::vk::DynamicState::DEPTH_BOUNDS);
		}

		if stencil_test_enable {
			if stencil_comare_dynamic {
				$dynamic_info.push($crate::ash::vk::DynamicState::STENCIL_COMPARE_MASK);
			}
			if stencil_write_dynamic {
				$dynamic_info.push($crate::ash::vk::DynamicState::STENCIL_WRITE_MASK);
			}
			if stencil_reference_dynamic {
				$dynamic_info.push($crate::ash::vk::DynamicState::STENCIL_REFERENCE);
			}
		}
	};

	(
		@ColorBlend($output_builder: expr, $dynamic_info: expr)
		$(logic_op: $logic_op: expr,)?
		attachments: [
			$(
				{ $($attachment_tt: tt)+ }
			),*
			$(,)?
		]
		$(, blend_constants: $blend_constants: expr)?
		$(,)?
	) => {
		let attachments = [
			$(
				$crate::color_blend_state_expr!($($attachment_tt)+)
			),*
		];

		#[allow(unused_assignments, unused_mut)]
		let mut logic_op: $crate::pipeline::params::BlendLogicOp = Default::default();
		$(
			logic_op = $logic_op;
		)?
		let (logic_op_enable, logic_op) = logic_op.into();

		#[allow(unused_assignments, unused_mut)]
		let mut blend_constants: Option<[f32; 4]> = Some([0.0; 4]);
		$(
			blend_constants = $blend_constants;
		)?
		let blend_constants_value = blend_constants.unwrap_or([0.0; 4]);

		let builder = $crate::ash::vk::PipelineColorBlendStateCreateInfo::builder()
			.logic_op_enable(logic_op_enable)
			.logic_op(logic_op)
			.attachments(
				$crate::util::transparent::Transparent::transmute_slice(&attachments)
			)
			.blend_constants(blend_constants_value)
		;

		$output_builder = $output_builder
			.color_blend_state(&builder)
		;

		if blend_constants.is_none() {
			$dynamic_info.push($crate::ash::vk::DynamicState::BLEND_CONSTANTS);
		}
	};

	(
		@Deps($output_builder: expr)
		layout: $layout: expr,
		render_pass: $render_pass: expr
		$(, subpass: $subpass: expr)?
		$(,)?
	) => {
		let layout: $crate::ash::vk::PipelineLayout = $layout.handle();
		let render_pass: $crate::ash::vk::RenderPass = $render_pass.handle();

		#[allow(unused_assignments, unused_mut)]
		let mut subpass: u32 = 0;
		$(
			subpass = $subpass;
		)?

		$output_builder = $output_builder
			.layout(layout)
			.render_pass(render_pass)
			.subpass(subpass)
		;
	};

	(
		let $create_info_variable_name: ident;

		Shaders {
			$($shaders_tt: tt)+
		}

		$(
			Tessellation {
				$($tessellation_tt: tt)+
			}
		)?

		$(
			Viewport {
				$($viewport_tt: tt)+
			}
		)?

		Rasterization {
			$($rasterization_tt: tt)+
		}

		$(
			Multisampling {
				$($multisampling_tt: tt)+
			}
		)?

		$(
			DepthStencil {
				$($depth_stencil_tt: tt)+
			}
		)?

		$(
			ColorBlend {
				$($color_blend_tt: tt)+
			}
		)?

		Deps {
			$($deps_tt: tt)+
		}
	) => {
		struct DynamicInfo {
			index: usize,
			array: [$crate::ash::vk::DynamicState; 9]
		}
		impl DynamicInfo {
			pub fn new() -> Self {
				DynamicInfo {
					index: 0,
					array: Default::default()
				}
			}

			pub fn push(&mut self, value: $crate::ash::vk::DynamicState) {
				debug_assert!(self.index < self.array.len());
				self.array[self.index] = value;
				self.index += 1;
			}

			pub fn as_slice(&self) -> &[$crate::ash::vk::DynamicState] {
				&self.array[..self.index]
			}
		}

		#[allow(unused_mut)]
		let mut dynamic_info = DynamicInfo::new();
		let mut builder = $crate::ash::vk::GraphicsPipelineCreateInfo::builder();

		$crate::create_graphics_pipeline!(
			@Shaders(builder)
			$($shaders_tt)+
		);

		$(
			$crate::create_graphics_pipeline!(
				@Tessellation(builder)
				$($tessellation_tt)*
			);
		)?

		$(
			$crate::create_graphics_pipeline!(
				@Viewport(builder, dynamic_info)
				$($viewport_tt)+
			);
		)?

		$crate::create_graphics_pipeline!(
			@Rasterization(builder, dynamic_info)
			$($rasterization_tt)+
		);

		$(
			$crate::create_graphics_pipeline!(
				@Multisampling(builder)
				$($multisampling_tt)+
			);
		)?

		$(
			$crate::create_graphics_pipeline!(
				@DepthStencil(builder, dynamic_info)
				$($depth_stencil_tt)+
			);
		)?

		$(
			$crate::create_graphics_pipeline!(
				@ColorBlend(builder, dynamic_info)
				$($color_blend_tt)+
			);
		)?

		$crate::create_graphics_pipeline!(
			@Deps(builder)
			$($deps_tt)+
		);

		let dynamic_state;
		if dynamic_info.as_slice().len() > 0 {
			dynamic_state = $crate::ash::vk::PipelineDynamicStateCreateInfo::builder()
				.dynamic_states(dynamic_info.as_slice())
			;
			builder = builder.dynamic_state(&dynamic_state);
		}

		// Final builder
		let $create_info_variable_name = builder;
	}
}

#[cfg(test)]
mod test {
	use ash::vk as vvk;

	#[test]
	#[ignore]
	fn test_graphics_pipeline_params() {
		offsetable_struct! {
			pub struct Vertex {
				position: [f32; 3],
				color: u32
			} repr(C) as VertexOffsets
		}
		#[repr(C)]
		struct Normal {
			normal: [f32; 3]
		}

		struct LayoutHandle;
		impl LayoutHandle {
			fn handle(&self) -> vvk::PipelineLayout {
				vvk::PipelineLayout::null()
			}
		}

		struct RenderPassHandle;
		impl RenderPassHandle {
			fn handle(&self) -> vvk::RenderPass {
				vvk::RenderPass::null()
			}
		}

		create_graphics_pipeline! {
			let create_info;

			Shaders {
				stages: []
				input: {
					Vertex {
						.position => layout(location = 0) in vec3 position;
						.color => layout(location = 2) in int color;
					}
					Normal {
						=> layout(location = 1) in vec3 normal;
					}
				}
				topology: vvk::PrimitiveTopology::TRIANGLE_LIST
			}

			Tessellation {
				patch_control_points: 0
			}

			Viewport {
				viewports: {
					[
						dynamic @ [0, 0, i32::MAX as u32, i32::MAX as u32]
					]
				}
			}

			Rasterization {
				polygon_mode: super::PolygonMode::LineDynamic(),
				depth_bias: super::DepthBias::Dynamic
			}

			Multisampling {
				samples: vvk::SampleCountFlags::TYPE_1
			}

			DepthStencil {
				depth: Default::default(),
				depth_bounds: super::DepthBoundsTest::Dynamic,
				stencil: super::StencilTest::Enabled {
					fail_op: Default::default(),
					pass_op: Default::default(),
					depth_fail_op: Default::default(),
					compare_op: Default::default(),
					compare_mask: None,
					write_mask: None,
					reference: None
				}
			}

			ColorBlend {
				logic_op: super::BlendLogicOp::default(),
				attachments: [
					{
						(S * SRC_ALPHA) ADD (D * ONE_MINUS_SRC_ALPHA)
							: (S * ONE) SUBTRACT (D * ZERO)
							& vvk::ColorComponentFlags::all()
					},
					{ disabled & vvk::ColorComponentFlags::all() }
				],
				blend_constants: None
			}

			Deps {
				layout: LayoutHandle,
				render_pass: RenderPassHandle
			}
		};

		macro_rules! dbg_it {
			(
				$base: expr;
				$(
					$field: ident[$len: literal]$({ $($rec_tt: tt)+ })?
				),+ $(,)?
			) => {
				$(
					if $base.$field == std::ptr::null() {
						eprintln!(
							"{} = null",
							stringify!($field)
						);
					} else {
						#[allow(unused_unsafe)]
						unsafe {
							for x in 0 .. $len {
								let field = *($base.$field.add(x));
								eprintln!(
									"[{:?}] {}[{}] = {:#?}",
									$base.$field.add(x),
									stringify!($field), x,
									field
								);
								$(
									dbg_it!(
										field;
										$($rec_tt)+
									);
								)?
							}
						}
						eprintln!("");
					}
				)+
			}
		}

		dbg!(create_info.flags);
		dbg_it!(
			create_info;

			p_stages[0],
			p_vertex_input_state[1]{p_vertex_binding_descriptions[2],p_vertex_attribute_descriptions[3]},
			p_input_assembly_state[1],

			p_tessellation_state[1],

			p_viewport_state[1]{p_viewports[2],p_scissors[2]},

			p_rasterization_state[1],

			p_multisample_state[1],

			p_depth_stencil_state[1],

			p_color_blend_state[1]{p_attachments[2]},

			p_dynamic_state[1]{p_dynamic_states[9]}
		);
	}
}
