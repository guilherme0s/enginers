use std::sync::Arc;

use ash::vk;

use super::instance::InstanceInner;

pub struct Device {
    #[allow(dead_code)]
    instance: Arc<InstanceInner>,
    raw: ash::Device,
    graphics_queue_family: u32,
}

impl Device {
    pub(super) fn new(
        instance: Arc<InstanceInner>,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self, crate::Error> {
        let queue_families = unsafe {
            instance
                .raw
                .get_physical_device_queue_family_properties(physical_device)
        };

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

        Ok(Self {
            instance,
            raw,
            graphics_queue_family,
        })
    }

    pub fn command_pool_create(&self) -> Result<vk::CommandPool, crate::Error> {
        let cmd_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(self.graphics_queue_family)
            .flags(vk::CommandPoolCreateFlags::empty());

        let vk_command_pool = unsafe {
            self.raw
                .create_command_pool(&cmd_pool_info, None)
                .map_err(|_| crate::Error::Unknown)?
        };

        Ok(vk_command_pool)
    }

    pub fn command_pool_destroy(&self, cmd_pool: vk::CommandPool) {
        unsafe {
            self.raw.destroy_command_pool(cmd_pool, None);
        }
    }

    pub fn command_buffer_allocate(
        &self,
        cmd_pool: vk::CommandPool,
    ) -> Result<vk::CommandBuffer, crate::Error> {
        let cmd_buf_alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(cmd_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let vk_command_buffer = unsafe {
            self.raw
                .allocate_command_buffers(&cmd_buf_alloc_info)
                .map_err(|_| crate::Error::Unknown)?
        }[0];

        Ok(vk_command_buffer)
    }

    pub fn command_buffer_begin(&self, cmd_buf: vk::CommandBuffer) -> Result<(), crate::Error> {
        let cmd_buf_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.raw
                .begin_command_buffer(cmd_buf, &cmd_buf_begin_info)
                .map_err(|_| crate::Error::Unknown)?
        };

        Ok(())
    }

    pub fn command_buffer_end(&self, cmd_buf: vk::CommandBuffer) -> Result<(), crate::Error> {
        unsafe {
            self.raw
                .end_command_buffer(cmd_buf)
                .map_err(|_| crate::Error::Unknown)?
        };

        Ok(())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.raw.destroy_device(None);
        }
    }
}
