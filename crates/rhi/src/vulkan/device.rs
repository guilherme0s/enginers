use ash::vk;

pub struct Device {
    raw: ash::Device,
}

impl Device {
    pub(super) fn new(
        instance: &super::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self, crate::Error> {
        let queue_families = unsafe {
            instance
                .raw
                .get_physical_device_queue_family_properties(physical_device)
        };

        println!("Found {} queue families", queue_families.len());

        let mut graphics_queue_family = 0;
        for (index, props) in queue_families.iter().enumerate() {
            if props.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_queue_family = index as u32;
                break;
            }
        }

        let queue_priority = [1.0f32];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_queue_family)
            .queue_priorities(&queue_priority);

        let extensions = [ash::khr::swapchain::NAME.as_ptr()];

        let features = vk::PhysicalDeviceFeatures::default();

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&extensions)
            .enabled_features(&features);

        let raw = unsafe {
            instance
                .raw
                .create_device(physical_device, &create_info, None)
                .map_err(|_| crate::Error::Unknown)?
        };

        Ok(Self { raw })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.raw.destroy_device(None);
        }
    }
}
