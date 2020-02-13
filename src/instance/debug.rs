use ash::vk::{DebugReportCallbackCreateInfoEXT, DebugReportFlagsEXT, DebugReportObjectTypeEXT, Bool32};
use std::os::raw::c_char;
use std::ffi::{c_void, CStr};
use std::borrow::Cow;

unsafe_enum_variants! {
	enum DebugCallbackInner {
		/// No debug callback will be registered.
		pub None,
		/// A default debug callback provided by Vulkayes will be registered.
		pub Default,
		/// A custom debug callback will be registered.
		{unsafe} pub Custom(DebugReportCallbackCreateInfoEXT)
	} as pub DebugCallback
}
impl Into<Option<DebugReportCallbackCreateInfoEXT>> for DebugCallback {
	fn into(self) -> Option<DebugReportCallbackCreateInfoEXT> {
		match self.0 {
			DebugCallbackInner::None => None,
			DebugCallbackInner::Default => {
				Some(
					DebugReportCallbackCreateInfoEXT::builder()
					.flags(DebugReportFlagsEXT::all())
					.pfn_callback(
						Some(default_debug_callback)
					)
					.build()
				)
			}
			DebugCallbackInner::Custom(info) => Some(info)
		}
	}
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
	_p_user_data: *mut c_void,
) -> Bool32 {
	let perf_optional = if flags == DebugReportFlagsEXT::PERFORMANCE_WARNING {
		"{PERF}"
	} else {
		""
	};

	let layer_prefix = CStr::from_ptr(p_layer_prefix).to_string_lossy();

	let object_optional = if object_type == DebugReportObjectTypeEXT::UNKNOWN {
		Cow::Borrowed("")
	} else {
		Cow::Owned(
			format!("<{:?} {}> ", object_type, object)
		)
	};

	let debug_message = CStr::from_ptr(p_message).to_string_lossy();

	let message = format!(
		"{}{} ({}:{}) {}{}",
		perf_optional,
		layer_prefix,
		location, message_code,
		object_optional,
		debug_message
	);

	match flags {
		DebugReportFlagsEXT::ERROR => log::error!("{}", message),
		DebugReportFlagsEXT::WARNING => log::warn!("{}", message),
		DebugReportFlagsEXT::INFORMATION => log::info!("{}", message),
		DebugReportFlagsEXT::DEBUG => log::debug!("{}", message),

		DebugReportFlagsEXT::PERFORMANCE_WARNING => log::warn!("{}", message),

		_ => log::error!("Message has multiple DebugReportFlagsEXT bits set: {}", message) // This should be unreachable
	}

	ash::vk::FALSE
}