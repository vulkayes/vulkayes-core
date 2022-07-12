use std::{
	borrow::Cow,
	ffi::{c_void, CStr},
	fmt::Write
};

use ash::vk::{
	self,
	Bool32,
	DebugUtilsMessageSeverityFlagsEXT,
	DebugUtilsMessageTypeFlagsEXT,
	DebugUtilsMessengerCallbackDataEXT,
	DebugUtilsMessengerCreateInfoEXT
};

unsafe_enum_variants! {
	#[derive(Debug)]
	enum DebugCallbackInner {
		/// No debug callback will be registered.
		pub None => { None },
		/// A default debug callback provided by Vulkayes will be registered.
		pub Default => {
			Some(
				DebugUtilsMessengerCreateInfoEXT::builder()
					.message_severity(
						DebugUtilsMessageSeverityFlagsEXT::VERBOSE
						| DebugUtilsMessageSeverityFlagsEXT::INFO
						| DebugUtilsMessageSeverityFlagsEXT::WARNING
						| DebugUtilsMessageSeverityFlagsEXT::ERROR
					)
					.message_type(
						DebugUtilsMessageTypeFlagsEXT::GENERAL
						| DebugUtilsMessageTypeFlagsEXT::VALIDATION
						| DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
					)
					.pfn_user_callback(Some(default_debug_callback))
					.build()
			)
		},
		/// A custom debug callback will be registered.
		{unsafe} pub Custom { info: DebugUtilsMessengerCreateInfoEXT } => { Some(info) }
	} as pub DebugCallback impl Into<Option<DebugUtilsMessengerCreateInfoEXT>>
}
impl Default for DebugCallback {
	fn default() -> Self {
		DebugCallback::None()
	}
}

/// Final message will look like this:
///
/// `{PERF} PREFIX (LOCATION:CODE) <OBJ_TYPE OBJ> MESSAGE`
pub unsafe extern "system" fn default_debug_callback(
	message_severity: DebugUtilsMessageSeverityFlagsEXT,
	message_type: DebugUtilsMessageTypeFlagsEXT,
	p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
	_user_data: *mut c_void
) -> Bool32 {
	let data = *p_callback_data;

	macro_rules! gib_str {
		($ptr: expr) => {
			if $ptr.is_null() {
				Cow::Borrowed("")
			} else {
				CStr::from_ptr($ptr).to_string_lossy()
			}
		};
	}

	let mut maybe_objects = String::new();
	if data.object_count > 0 {
		let objects = std::slice::from_raw_parts(
			data.p_objects,
			data.object_count as usize
		);
		for object in objects {
			let _ = write!(
				&mut maybe_objects,
				"<{:?} 0x{:x} \"{}\"> ",
				object.object_type,
				object.object_handle,
				gib_str!(object.p_object_name)
			);
		}
	}

	let message = format!(
		"[{:?}] {}({}) {}{}",
		message_type,
		gib_str!(data.p_message_id_name),
		data.message_id_number,
		maybe_objects,
		gib_str!(data.p_message)
	);

	let log_level = if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::VERBOSE) {
		log::Level::Debug
	} else if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::INFO) {
		log::Level::Info
	} else if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::WARNING) {
		log::Level::Warn
	} else if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::ERROR) {
		log::Level::Error
	} else {
		log::Level::Trace
	};

	log::log!(log_level, "{}", message);

	vk::FALSE
}
