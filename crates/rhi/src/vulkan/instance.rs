use std::{ffi::CString, sync::Arc};

use ash::vk;

pub struct Instance(Arc<InstanceInner>);

pub(crate) struct InstanceInner {
    _entry: ash::Entry,
    raw: ash::Instance,
    debug_utils: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl Instance {
    pub fn new() -> Result<Self, crate::Error> {
        let entry = unsafe { ash::Entry::load().map_err(|_| crate::Error::Unknown)? };

        let app_name = CString::new("MyEngine").unwrap();
        let engine_name = CString::new("MyEngine").unwrap();

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(1)
            .engine_name(&engine_name)
            .engine_version(1)
            .api_version(vk::API_VERSION_1_3);

        let mut extensions = vec![ash::khr::surface::NAME.as_ptr()];

        #[cfg(debug_assertions)]
        extensions.push(ash::ext::debug_utils::NAME.as_ptr());

        let mut instance_layers = Vec::new();

        #[cfg(debug_assertions)]
        instance_layers.push(
            std::ffi::CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")
                .unwrap()
                .as_ptr(),
        );

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&instance_layers);

        let raw = match unsafe { entry.create_instance(&create_info, None) } {
            Ok(raw) => raw,
            Err(vk::Result::ERROR_LAYER_NOT_PRESENT) => {
                let create_info = vk::InstanceCreateInfo::default()
                    .application_info(&app_info)
                    .enabled_extension_names(&extensions);

                unsafe {
                    entry
                        .create_instance(&create_info, None)
                        .map_err(|_| crate::Error::Unknown)?
                }
            }
            Err(_) => return Err(crate::Error::Unknown),
        };

        let (debug_utils, debug_messenger) = Self::setup_debug_messenger(&entry, &raw)?;

        let inner = Arc::new(InstanceInner {
            _entry: entry,
            raw,
            debug_utils,
            debug_messenger,
        });

        Ok(Self(inner))
    }

    pub fn create_device(&self) -> Result<super::Device, crate::Error> {
        let physical_devices = match unsafe { self.0.raw.enumerate_physical_devices() } {
            Ok(devices) => devices,
            Err(_) => Vec::new(),
        };
        let physical_device = physical_devices[0];

        let queue_families = unsafe {
            self.0
                .raw
                .get_physical_device_queue_family_properties(physical_device)
        };

        let graphics_queue_family = queue_families
            .iter()
            .enumerate()
            .find(|(_, q)| q.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|(i, _)| i as u32)
            .unwrap();

        let queue_priority = [1.0f32];

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_queue_family)
            .queue_priorities(&queue_priority);

        let device_features = vk::PhysicalDeviceFeatures::default();

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_features(&device_features);

        let raw = unsafe {
            self.0
                .raw
                .create_device(physical_device, &create_info, None)
                .map_err(|_| crate::Error::Unknown)?
        };

        Ok(super::Device { raw })
    }

    fn setup_debug_messenger(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Result<
        (
            Option<ash::ext::debug_utils::Instance>,
            Option<vk::DebugUtilsMessengerEXT>,
        ),
        crate::Error,
    > {
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

        Ok((Some(loader), messenger))
    }
}

impl Drop for InstanceInner {
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
    _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = unsafe { std::ffi::CStr::from_ptr((*callback_data).p_message) };

    println!("{:?} {:?}", message_type, message);

    vk::FALSE
}
