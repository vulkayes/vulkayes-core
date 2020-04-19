use std::{
	borrow::Cow,
	ffi::{c_void, CStr},
	os::raw::c_char
};

use ash::vk::{
	self,
	Bool32,
	DebugReportCallbackCreateInfoEXT,
	DebugReportFlagsEXT,
	DebugReportObjectTypeEXT
};

unsafe_enum_variants! {
	#[derive(Debug)]
	enum DebugCallbackInner {
		/// No debug callback will be registered.
		pub None => { None },
		/// A default debug callback provided by Vulkayes will be registered.
		pub Default => {
			Some(
				DebugReportCallbackCreateInfoEXT::builder()
					.flags(DebugReportFlagsEXT::all())
					.pfn_callback(Some(default_debug_callback))
					.build()
			)
		},
		/// A custom debug callback will be registered.
		{unsafe} pub Custom { info: DebugReportCallbackCreateInfoEXT } => { Some(info) }
	} as pub DebugCallback impl Into<Option<DebugReportCallbackCreateInfoEXT>>
}
impl Default for DebugCallback {
	fn default() -> Self {
		DebugCallback::None()
	}
}

/// Final message will look like this:
///
/// `{PERF} PREFIX (LOCATION:CODE) <OBJ_TYPE OBJ> MESSAGE`
unsafe extern "system" fn default_debug_callback(
	flags: DebugReportFlagsEXT,
	object_type: DebugReportObjectTypeEXT,
	object: u64,
	location: usize,
	message_code: i32,
	p_layer_prefix: *const c_char,
	p_message: *const c_char,
	_p_user_data: *mut c_void
) -> Bool32 {
	let perf_optional = if flags == DebugReportFlagsEXT::PERFORMANCE_WARNING {
		"{PERF} "
	} else {
		""
	};

	let layer_prefix = CStr::from_ptr(p_layer_prefix).to_string_lossy();

	let object_optional = if object_type == DebugReportObjectTypeEXT::UNKNOWN {
		Cow::Borrowed("")
	} else {
		Cow::Owned(format!("<{:?} {}> ", object_type, object))
	};

	let debug_message = CStr::from_ptr(p_message).to_string_lossy();

	let message = format!(
		"{}{} ({}:{}) {}{}",
		perf_optional, layer_prefix, location, message_code, object_optional, debug_message
	);

	match flags {
		DebugReportFlagsEXT::ERROR => log::error!("{}", message),
		DebugReportFlagsEXT::WARNING => log::warn!("{}", message),
		DebugReportFlagsEXT::INFORMATION => log::info!("{}", message),
		DebugReportFlagsEXT::DEBUG => log::debug!("{}", message),

		DebugReportFlagsEXT::PERFORMANCE_WARNING => log::warn!("{}", message),

		_ => log::error!(
			"Message has multiple DebugReportFlagsEXT bits set: {}",
			message
		) // This should be unreachable
	}

	vk::FALSE
}
