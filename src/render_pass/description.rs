/// Generates render pass attachment descriptions and subpass descriptions.
///
/// The syntax is:
/// ```
/// # use vulkayes_core::render_pass_description;
/// # use vulkayes_core::render_pass::params::{AttachmentOps, AttachmentDescription, SubpassDescriptionHolder, AttachmentReference};
///
/// let attachments: [AttachmentDescription; 2];
/// let holders: (SubpassDescriptionHolder<[AttachmentReference; 1], [AttachmentReference; 1], [u32; 1]>);
///
/// let (attachments, holders) = render_pass_description! {
/// 	Attachments {
/// 		// ident for the unused meta-attachment
/// 		UNUSED,
///
/// 		// name of the attachment, has to be unique across the whole macro to avoid collisions inside the macro
/// 		Foo {
/// 			format = R8_UINT, // any associated constant of vk::Format
/// 			ops = AttachmentOps::Color {
/// 				load: vk::AttachmentLoadOp::CLEAR,
/// 				store: vk::AttachmentStoreOp::DONT_CARE
/// 			},
/// 			layouts = UNDEFINED => COLOR_ATTACHMENT_OPTIMAL, // initial layout (vk::ImageLayout) and final layout (ImageLayoutFinal)
/// 			samples = TYPE_2, // optional, any associated constant of vk::SampleCountFlags
/// 			may_alias = true // optional, controls the vk::AttachmentDescriptionFlags::MAY_ALIAS flag
/// 		}
///
/// 		Bar {
/// 			format = D16_UNORM_S8_UINT,
/// 			ops = AttachmentOps::DepthStencil {
/// 				depth_load: vk::AttachmentLoadOp::CLEAR,
/// 				depth_store: vk::AttachmentStoreOp::DONT_CARE,
/// 					stencil_load: vk::AttachmentLoadOp::LOAD,
/// 				stencil_store: vk::AttachmentStoreOp::STORE
/// 			},
/// 			layouts = UNDEFINED => DEPTH_STENCIL_ATTACHMENT_OPTIMAL
/// 		}
///
/// 		// etc.
/// 	}
/// 	Subpasses {
/// 		// name of the subpass, has to be unique across the whole macro to avoid collisions inside the macro
/// 		First {
/// 			// optional, specifies input attachments
/// 			input = [
/// 				@Foo // uses attachment named Foo with layout COLOR_ATTACHMENT_OPTIMAL (final_layout)
/// 			]
///
/// 			// optional, specifies color attachments
/// 			color = [
/// 				@Foo<GENERAL> // uses with layout GENERAL
/// 			]
///
/// 			// optional, can only be specified if color attachments are also specified,
/// 			// specifies resolve attachments, size has to match color attachments
/// 			resolve = [
/// 				@UNUSED // unused resolve attachment
/// 			]
///
/// 			// optional, specifies depth stencil attachment with layout DEPTH_STENCIL_ATTACHMENT_OPTIMAL
/// 			depth_stencil = @Bar<DEPTH_STENCIL_ATTACHMENT_OPTIMAL>
///
/// 			// optional, specifies attachments to preserve
/// 			preserve = [
/// 				@Foo // not valid
/// 			]
/// 		}
///
/// 		// etc.
/// 	}
/// };
/// ```
#[macro_export]
macro_rules! render_pass_description {
	(
		Attachments {
			$unused: ident,
			$(
				$att_name: ident {
					format = $format: ident,
					ops = $ops: expr,
					layouts = $initial_layout: ident => $final_layout: ident
					$(, samples = $samples: ident)?
					$(, may_alias = $may_alias: expr)?
					$(,)?
				}
			)*
		}
		Subpasses {
			$(
				$sub_name: ident {
					$(
						input = [
							$(
								@$input_name: ident $(<$input_layout: ident>)?
							),+ $(,)?
						]
					)?
					$(
						color = [
							$(
								@$color_name: ident $(<$color_layout: ident>)?
							),+ $(,)?
						]
						$(
							resolve = [
								$(
									@$resolve_name: ident $(<$resolve_layout: ident>)?
								),+ $(,)?
							]
						)?
					)?
					$(
						depth_stencil = @$ds_name: ident $(<$ds_layout: ident>)?
					)?
					$(
						preserve = [
							$(
								@$preserve_name: ident
							),+ $(,)?
						]
					)?
				}
			)+
		}
	) => {
		{
			#[allow(unused_imports)]
			use $crate::ash::vk;
			#[allow(unused_imports)]
			use $crate::render_pass::params::{AttachmentDescription, AttachmentReference, SubpassDescriptionHolder};
			#[allow(unused_imports)]
			use $crate::resource::image::layout::{ImageLayoutFinal, ImageLayoutAttachment};

			// Create attachment descriptions.
			// Each description is a tuple of (index, description)
			#[allow(non_snake_case)]
			#[allow(unused_variables)]
			let $unused = (
				None::<u32>,
				unsafe {
					AttachmentDescription::from_raw(
						vk::AttachmentDescription::builder().final_layout(vk::ImageLayout::GENERAL)
					)
				}
			);
			let counter: u32 = 0;
			$(
				#[allow(non_snake_case)]
				let $att_name = {
					#[allow(unused_variables)]
					let may_alias = false;
					$(let may_alias: bool = $may_alias;)?

					#[allow(unused_variables)]
					let samples = vk::SampleCountFlags::TYPE_1;
					$(let samples = vk::SampleCountFlags::$samples;)?

					(
						Some(counter),
						AttachmentDescription::new(
							may_alias,
							vk::Format::$format,
							samples,
							$ops,
							vk::ImageLayout::$initial_layout,
							ImageLayoutFinal::$final_layout
						)
					)
				};
				#[allow(unused_variables)]
				let counter = counter + 1;
			)*

			// Create subpass descriptions
			$(
				#[allow(non_snake_case)]
				let $sub_name: SubpassDescriptionHolder<_, _, _> = {
					let input_attachments = render_pass_description!(
						__INNER_attachment_references
						$(
							$($input_name $($input_layout)?),+
						)?
					);

					let color_attachments = render_pass_description!(
						__INNER_attachment_references
						$(
							$($color_name $($color_layout)?),+
						)?
					);
					let resolve_attachments = render_pass_description!(
						__INNER_attachment_references
						$(
							$(
								$($resolve_name $($resolve_layout)?),+
							)?
						)?
					);
					let color_resolve_attachments = color_attachments.map(|c| (c, resolve_attachments));

					#[allow(unused_variables)]
					let depth_stencil_attachment: Option<AttachmentReference> = None;
					$(
						let depth_stencil_attachment = Some(
							render_pass_description!(
								__INNER_attachment_reference
								$ds_name $($ds_layout)?
							)
						);
					)?

					#[allow(unused_variables)]
					let preserve_attachments: Option<[u32; 0]> = None;
					$(
						let preserve_attachments = Some([
							$(
								$preserve_name.0.expect("Preserved attachment must not be unused")
							),+
						]);
					)?

					SubpassDescriptionHolder {
						input_attachments,
						color_resolve_attachments,
						depth_stencil_attachment,
						preserve_attachments
					}
				};
			)+

			(
				[$($att_name.1),*],
				($($sub_name),+)
			)
		}
	};

	(
		__INNER_attachment_references
		$(
			$(
				$name: ident $($layout: ident)?
			),+
		)?
	) => {
		{
			#[allow(unused_variables)]
			let attachments: Option<[AttachmentReference; 0]> = None;
			$(
				let attachments = Some([
					$(
						render_pass_description!(
							__INNER_attachment_reference
							$name $($layout)?
						)
					),+
				]);
			)?

			attachments
		}
	};

	(
		__INNER_attachment_reference
		$name: ident $($layout: ident)?
	) => {
		{
			#[allow(unused_variables)]
			let layout: Result<ImageLayoutAttachment, _> = std::convert::TryInto::try_into($name.1.final_layout);
			$(
				let layout: Result<
					ImageLayoutAttachment,
					<ImageLayoutAttachment as std::convert::TryFrom<vk::ImageLayout>>::Error
				> = Ok(ImageLayoutAttachment::$layout);
			)?
			let layout: ImageLayoutAttachment = match layout {
				Ok(v) => v,
				Err(err) => panic!("Could not convert {}.final_layout into ImageLayoutAttachment: {}", stringify!($name), err)
			};

			AttachmentReference::new(
				$name.0,
				layout
			)
		}
	};
}

#[cfg(test)]
mod test {
	use ash::vk;

	use crate::render_pass::params::AttachmentOps;

	#[test]
	fn test_render_pass_description() {
		let (attachments, holders) = render_pass_description!(
			Attachments {
				UNUSED,
				Foo {
					format = R8_UNORM,
					ops = AttachmentOps::Color {
						load: vk::AttachmentLoadOp::CLEAR,
						store: vk::AttachmentStoreOp::DONT_CARE
					},
					layouts = UNDEFINED => COLOR_ATTACHMENT_OPTIMAL,
					samples = TYPE_2,
					may_alias = true
				}
				Bar {
					format = R8_UINT,
					ops = AttachmentOps::Color {
						load: vk::AttachmentLoadOp::CLEAR,
						store: vk::AttachmentStoreOp::DONT_CARE
					},
					layouts = PREINITIALIZED => SHADER_READ_ONLY_OPTIMAL,
					samples = TYPE_1
				}
				Baz {
					format = D16_UNORM_S8_UINT,
					ops = AttachmentOps::DepthStencil {
						depth_load: vk::AttachmentLoadOp::CLEAR,
						depth_store: vk::AttachmentStoreOp::DONT_CARE,
						stencil_load: vk::AttachmentLoadOp::LOAD,
						stencil_store: vk::AttachmentStoreOp::STORE
					},
					layouts = UNDEFINED => DEPTH_STENCIL_ATTACHMENT_OPTIMAL
				}
			}
			Subpasses {
				First {
					color = [@Foo, @UNUSED]
					resolve = [@Bar<GENERAL>, @UNUSED]
					depth_stencil = @Baz<DEPTH_STENCIL_ATTACHMENT_OPTIMAL>
				}
				Second {
					input = [@Bar<COLOR_ATTACHMENT_OPTIMAL>]
					preserve = [@Foo]
				}
			}
		);

		println!("{:#?}", attachments);
		println!("{:#?}", holders);

		{
			assert_eq!(attachments.len(), 3);

			let expected = [
				vk::AttachmentDescription::builder()
					.flags(vk::AttachmentDescriptionFlags::MAY_ALIAS)
					.format(vk::Format::R8_UNORM)
					.samples(vk::SampleCountFlags::TYPE_2)
					.load_op(vk::AttachmentLoadOp::CLEAR)
					.store_op(vk::AttachmentStoreOp::DONT_CARE)
					.initial_layout(vk::ImageLayout::UNDEFINED)
					.final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
					.build(),
				vk::AttachmentDescription::builder()
					.format(vk::Format::R8_UINT)
					.samples(vk::SampleCountFlags::TYPE_1)
					.load_op(vk::AttachmentLoadOp::CLEAR)
					.store_op(vk::AttachmentStoreOp::DONT_CARE)
					.initial_layout(vk::ImageLayout::PREINITIALIZED)
					.final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
					.build(),
				vk::AttachmentDescription::builder()
					.format(vk::Format::D16_UNORM_S8_UINT)
					.samples(vk::SampleCountFlags::TYPE_1)
					.load_op(vk::AttachmentLoadOp::CLEAR)
					.store_op(vk::AttachmentStoreOp::DONT_CARE)
					.stencil_load_op(vk::AttachmentLoadOp::LOAD)
					.stencil_store_op(vk::AttachmentStoreOp::STORE)
					.initial_layout(vk::ImageLayout::UNDEFINED)
					.final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
					.build()
			];

			// No eq :(, but repr(C)
			unsafe {
				assert_eq!(
					std::slice::from_raw_parts(
						&attachments as *const _ as *const u8,
						std::mem::size_of_val(&attachments)
					),
					std::slice::from_raw_parts(
						&expected as *const _ as *const u8,
						std::mem::size_of_val(&expected)
					)
				);
			}
		}
	}
}