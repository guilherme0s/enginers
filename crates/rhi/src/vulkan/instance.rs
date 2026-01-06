use std::ffi::CStr;

use ash::vk;

static VK_LAYER_KHRONOS_VALIDATION_NAME: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };

pub struct Instance {
    /// Vulkan loader entry point.
    ///
    /// This MUST be kept alive for the entire lifetime of the instance, as it contains
    /// the function pointers that `ash::Instance` uses. If this is dropped first,
    /// we'd have dangling function pointers.
    #[allow(dead_code)]
    entry: ash::Entry,

    raw: ash::Instance,
    debug_utils: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl Instance {
    pub fn new() -> Result<Self, crate::Error> {
        let entry = unsafe { ash::Entry::load().unwrap() };

        let app_info = vk::ApplicationInfo::default()
            .application_name(CStr::from_bytes_with_nul(b"\0").unwrap())
            .application_version(0)
            .engine_name(CStr::from_bytes_with_nul(b"\0").unwrap())
            .engine_version(0)
            .api_version(vk::API_VERSION_1_3);

        let mut extensions = Vec::new();

        #[cfg(debug_assertions)]
        extensions.push(ash::ext::debug_utils::NAME.as_ptr());

        let mut instance_layers = Vec::new();

        #[cfg(debug_assertions)]
        instance_layers.push(VK_LAYER_KHRONOS_VALIDATION_NAME.as_ptr());

        let mut create_info = vk::InstanceCreateInfo::default()
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&instance_layers)
            .application_info(&app_info);

        let raw = match unsafe { entry.create_instance(&create_info, None) } {
            Ok(raw) => raw,
            Err(vk::Result::ERROR_LAYER_NOT_PRESENT) => {
                create_info = create_info.enabled_layer_names(&[]);

                unsafe {
                    entry
                        .create_instance(&create_info, None)
                        .map_err(|_| crate::Error::Unknown)?
                }
            }
            Err(_) => return Err(crate::Error::Unknown),
        };

        let (debug_utils, debug_messenger) = Self::setup_debug_messenger(&entry, &raw);

        Ok(Self {
            entry,
            raw,
            debug_utils,
            debug_messenger,
        })
    }

    fn setup_debug_messenger(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> (
        Option<ash::ext::debug_utils::Instance>,
        Option<vk::DebugUtilsMessengerEXT>,
    ) {
        let loader = ash::ext::debug_utils::Instance::new(entry, instance);
        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let messenger = unsafe { loader.create_debug_utils_messenger(&create_info, None).ok() };

        (Some(loader), messenger)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            if let (Some(utils), Some(messenger)) = (&self.debug_utils, self.debug_messenger) {
                utils.destroy_debug_utils_messenger(messenger, None);
            }
            self.raw.destroy_instance(None);
        }
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    _: vk::DebugUtilsMessageSeverityFlagsEXT,
    _: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = unsafe { std::ffi::CStr::from_ptr((*data).p_message) };
    println!("{:?}", message);
    vk::FALSE
}
