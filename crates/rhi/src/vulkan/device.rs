use std::sync::Arc;

use ash::vk;

use crate::RHIDevice;

use super::{CommandEncoder, instance::InstanceInner};

pub struct Swapchain {
    raw: vk::SwapchainKHR,
    image_views: Vec<vk::ImageView>,
}

pub struct Device {
    instance: Arc<InstanceInner>,
    raw: ash::Device,
    graphics_queue_family: u32,
    #[allow(unused)]
    graphics_queue: vk::Queue,
    physical_device: vk::PhysicalDevice,
    swapchain_loader: ash::khr::swapchain::Device,
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

        let graphics_queue = unsafe { raw.get_device_queue(graphics_queue_family, 0) };

        let swapchain_loader = ash::khr::swapchain::Device::new(&instance.raw, &raw);

        Ok(Self {
            instance,
            raw,
            graphics_queue_family,
            graphics_queue,
            physical_device,
            swapchain_loader,
        })
    }

    pub fn swapchain_create(
        &self,
        surface: &super::Surface,
        width: u32,
        height: u32,
    ) -> Result<Swapchain, crate::Error> {
        let surface_loader = &self.instance.surface_loader;

        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(self.physical_device, surface.raw)
                .map_err(|_| crate::Error::Unknown)?
        };

        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(self.physical_device, surface.raw)
                .map_err(|_| crate::Error::Unknown)?
        };

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(self.physical_device, surface.raw)
                .map_err(|_| crate::Error::Unknown)?
        };

        let surface_format = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_UNORM
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .copied()
            .unwrap_or(formats[0]);

        let present_mode = present_modes
            .iter()
            .copied()
            .find(|&m| m == vk::PresentModeKHR::MAILBOX)
            .or_else(|| {
                present_modes
                    .iter()
                    .copied()
                    .find(|&m| m == vk::PresentModeKHR::FIFO)
            })
            .unwrap();

        let extent = if surface_capabilities.current_extent.width != u32::MAX {
            surface_capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(
                    surface_capabilities.min_image_extent.width,
                    surface_capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    surface_capabilities.min_image_extent.height,
                    surface_capabilities.max_image_extent.height,
                ),
            }
        };

        let mut image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0 {
            image_count = image_count.min(surface_capabilities.max_image_count);
        }

        let swap_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.raw)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let vk_swapchain = unsafe {
            self.swapchain_loader
                .create_swapchain(&swap_create_info, None)
                .map_err(|_| crate::Error::Unknown)?
        };

        let images = unsafe {
            self.swapchain_loader
                .get_swapchain_images(vk_swapchain)
                .map_err(|_| crate::Error::Unknown)?
        };

        let mut image_views = Vec::with_capacity(images.len());

        for &image in &images {
            let image_view_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .level_count(1)
                        .layer_count(1),
                );

            let image_view = unsafe {
                self.raw
                    .create_image_view(&image_view_info, None)
                    .map_err(|_| crate::Error::Unknown)?
            };

            image_views.push(image_view);
        }

        Ok(Swapchain {
            raw: vk_swapchain,
            image_views,
        })
    }

    pub fn swapchain_destroy(&self, swapchain: Swapchain) {
        unsafe {
            for image_view in swapchain.image_views {
                self.raw.destroy_image_view(image_view, None);
            }

            self.swapchain_loader.destroy_swapchain(swapchain.raw, None);
        }
    }
}

impl RHIDevice for Device {
    type CommandEncoder = CommandEncoder;

    fn create_command_encoder(&self) -> Result<Self::CommandEncoder, crate::Error> {
        let cmd_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(self.graphics_queue_family)
            .flags(vk::CommandPoolCreateFlags::empty());

        let cmd_pool = unsafe {
            self.raw
                .create_command_pool(&cmd_pool_info, None)
                .map_err(|_| crate::Error::Unknown)?
        };

        let cmd_buffer_alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(cmd_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let cmd_buffer = unsafe {
            self.raw
                .allocate_command_buffers(&cmd_buffer_alloc_info)
                .map_err(|_| crate::Error::Unknown)?
        }[0];

        let cmd_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.raw
                .begin_command_buffer(cmd_buffer, &cmd_buffer_begin_info)
                .map_err(|_| crate::Error::Unknown)?
        };

        Ok(CommandEncoder {
            cmd_pool,
            cmd_buffer,
            device: self.raw.clone(),
        })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.raw.destroy_device(None);
        }
    }
}
