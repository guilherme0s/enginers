use std::ffi::CStr;
use std::sync::Arc;

use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

/// Canonical name of the Khronos validation layer.
static VK_LAYER_KHRONOS_VALIDATION_NAME: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };

/// Environment variable used to detect a Wayland session on Linux.
static WAYLAND_DISPLAY_ENVIRONMENT_VAR_NAME: &str = "WAYLAND_DISPLAY";

pub(super) struct InstanceInner {
    /// Vulkan loader entry point.
    ///
    /// This MUST outlive `ash::Instance`, because the instance internally
    /// stores function pointers resolved via this entry.
    pub entry: ash::Entry,

    /// Vulkan instance handle representing the application's connection
    /// to the Vulkan driver.
    pub raw: ash::Instance,

    pub surface_loader: ash::khr::surface::Instance,

    debug_utils: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

#[derive(Clone)]
pub struct Instance {
    pub(super) inner: Arc<InstanceInner>,
}

impl Instance {
    pub fn new() -> Result<Self, crate::Error> {
        let entry = unsafe { ash::Entry::load().map_err(|_| crate::Error::Unknown)? };

        let app_info = vk::ApplicationInfo::default()
            .application_name(c"")
            .application_version(0)
            .engine_name(c"")
            .engine_version(0)
            .api_version(vk::API_VERSION_1_3);

        let mut extensions = Vec::new();

        #[cfg(target_os = "linux")]
        {
            use std::env;

            // Detect active display server and load appropriate WSI extension
            // Only one WSI extension can be enabled at a time
            extensions.push(
                if env::var_os(WAYLAND_DISPLAY_ENVIRONMENT_VAR_NAME).is_some() {
                    ash::khr::wayland_surface::NAME.as_ptr()
                } else {
                    ash::khr::xlib_surface::NAME.as_ptr()
                },
            );
        }

        extensions.push(ash::khr::surface::NAME.as_ptr());

        #[cfg(debug_assertions)]
        extensions.push(ash::ext::debug_utils::NAME.as_ptr());

        let mut instance_layers = Vec::new();

        #[cfg(debug_assertions)]
        instance_layers.push(VK_LAYER_KHRONOS_VALIDATION_NAME.as_ptr());

        let create_info = vk::InstanceCreateInfo::default()
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&instance_layers)
            .application_info(&app_info);

        let raw = match unsafe { entry.create_instance(&create_info, None) } {
            Ok(raw) => raw,
            Err(vk::Result::ERROR_LAYER_NOT_PRESENT) => unsafe {
                entry
                    .create_instance(&create_info.enabled_layer_names(&[]), None)
                    .map_err(|_| crate::Error::Unknown)?
            },
            Err(_) => return Err(crate::Error::Unknown),
        };

        let (debug_utils, debug_messenger) = Self::setup_debug_messenger(&entry, &raw);

        let surface_loader = ash::khr::surface::Instance::new(&entry, &raw);

        let inner = Arc::new(InstanceInner {
            entry,
            raw,
            surface_loader,
            debug_utils,
            debug_messenger,
        });

        Ok(Self { inner })
    }

    pub fn create_surface(
        &self,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Result<super::Surface, crate::Error> {
        let raw = match (display_handle, window_handle) {
            #[cfg(target_os = "linux")]
            (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(surface)) => self
                .create_surface_from_wayland(display.display.as_ptr(), surface.surface.as_ptr())?,

            _ => return Err(crate::Error::Unknown),
        };

        Ok(super::Surface {
            raw,
            instance: self.inner.clone(),
        })
    }

    pub fn create_device(&self) -> Result<super::Device, crate::Error> {
        let physical_devices = unsafe {
            self.inner
                .raw
                .enumerate_physical_devices()
                .map_err(|_| crate::Error::Unknown)?
        };

        if physical_devices.is_empty() {
            eprintln!(
                "Could not find a compatible Vulkan device or driver.\n\
            Make sure your video card supports Vulkan and try updating your video driver."
            );
            return Err(crate::Error::Unknown);
        }

        struct DeviceInfo {
            raw: vk::PhysicalDevice,
            device_type: vk::PhysicalDeviceType,
            original_index: usize,
        }

        let mut device_infos: Vec<_> = physical_devices
            .iter()
            .enumerate()
            .map(|(index, &device)| {
                let props = unsafe { self.inner.raw.get_physical_device_properties(device) };
                DeviceInfo {
                    raw: device,
                    device_type: props.device_type,
                    original_index: index,
                }
            })
            .collect();

        // Priority: Discrete GPU > Integrated GPU > CPU > others
        device_infos.sort_by(|a, b| {
            use vk::PhysicalDeviceType;

            if a.device_type == b.device_type {
                a.original_index.cmp(&b.original_index)
            } else if a.device_type == PhysicalDeviceType::DISCRETE_GPU {
                std::cmp::Ordering::Less
            } else if b.device_type == PhysicalDeviceType::DISCRETE_GPU {
                std::cmp::Ordering::Greater
            } else if a.device_type == PhysicalDeviceType::INTEGRATED_GPU {
                std::cmp::Ordering::Less
            } else if b.device_type == PhysicalDeviceType::INTEGRATED_GPU {
                std::cmp::Ordering::Greater
            } else if a.device_type == PhysicalDeviceType::CPU {
                std::cmp::Ordering::Less
            } else if b.device_type == PhysicalDeviceType::CPU {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Greater
            }
        });

        // TODO: Decide whether CPU devices should be excluded entirely.

        let chosen_device = device_infos[0].raw;

        Ok(super::Device::new(self.inner.clone(), chosen_device)?)
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

        // Failure is non-fatal; the engine can run without it.
        let messenger = unsafe { loader.create_debug_utils_messenger(&create_info, None).ok() };

        (Some(loader), messenger)
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
    _: vk::DebugUtilsMessageSeverityFlagsEXT,
    _: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = unsafe { std::ffi::CStr::from_ptr((*data).p_message) };
    println!("{:?}", message);
    vk::FALSE
}
