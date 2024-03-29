use ash::vk;

vk_enum_subset! {
	/// Enum for image layout that can be used in final or new layout position.
	///
	/// This includes all layouts but `UNDEFINED` and `PREINITIALIZED`.
	pub enum ImageLayoutFinal {
		GENERAL,
		COLOR_ATTACHMENT_OPTIMAL,
		DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
		DEPTH_STENCIL_READ_ONLY_OPTIMAL,
		SHADER_READ_ONLY_OPTIMAL,
		TRANSFER_SRC_OPTIMAL,
		TRANSFER_DST_OPTIMAL,

		DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL,
		DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL,
		DEPTH_ATTACHMENT_OPTIMAL,
		DEPTH_READ_ONLY_OPTIMAL,
		STENCIL_ATTACHMENT_OPTIMAL,
		STENCIL_READ_ONLY_OPTIMAL,

		PRESENT_SRC_KHR,
		SHARED_PRESENT_KHR,

		SHADING_RATE_OPTIMAL_NV,
		FRAGMENT_DENSITY_MAP_OPTIMAL_EXT
	} impl Into<vk::ImageLayout>
}

vk_enum_subset! {
	/// Enum for image layout that can be used in render pass attachment position.
	pub enum ImageLayoutAttachment {
		GENERAL,
		COLOR_ATTACHMENT_OPTIMAL,
		DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
		DEPTH_STENCIL_READ_ONLY_OPTIMAL,
		SHADER_READ_ONLY_OPTIMAL,
		TRANSFER_SRC_OPTIMAL,
		TRANSFER_DST_OPTIMAL,

		DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL,
		DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL,

		SHARED_PRESENT_KHR,

		SHADING_RATE_OPTIMAL_NV,
		FRAGMENT_DENSITY_MAP_OPTIMAL_EXT
	} impl Into<vk::ImageLayout>
}

pub type ImageLayoutSource = vk::ImageLayout;

vk_enum_subset! {
	/// Enum for image layout that can be used as destination of image transfer operations.
	pub enum ImageLayoutDestination {
		GENERAL,
		TRANSFER_DST_OPTIMAL,
		SHARED_PRESENT_KHR
	} impl Into<vk::ImageLayout>
}

pub type ImageLayoutClearColorImage = ImageLayoutDestination;

vk_enum_subset! {
	/// Enum for image layout that can be used as a sampled image in shaders.
	pub enum ImageLayoutSampled {
		GENERAL,
		DEPTH_STENCIL_READ_ONLY_OPTIMAL,
		SHADER_READ_ONLY_OPTIMAL,

		DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL,
		DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL,

		SHARED_PRESENT_KHR
	} impl Into<vk::ImageLayout>
}

pub type ImageLayoutInputAttachment = ImageLayoutSampled;
