use std::sync::Arc;

use ash::vk;

use super::{InstanceInner, Surface, Swapchain};

pub struct Device(pub(super) Arc<DeviceInner>);

pub struct DeviceInner {
    pub(super) instance: Arc<InstanceInner>,
    pub(super) raw: ash::Device,
    pub(super) physical_device: vk::PhysicalDevice,
}

impl Device {
    pub fn create_swapchain(
        &self,
        surface: &Surface,
        desc: &crate::SwapchainDescriptor,
    ) -> Result<Swapchain, crate::Error> {
        let capabilities = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(self.0.physical_device, surface.raw)
                .map_err(|_| crate::Error::Unknown)?
        };

        let formats = unsafe {
            surface
                .loader
                .get_physical_device_surface_formats(self.0.physical_device, surface.raw)
                .map_err(|_| crate::Error::Unknown)?
        };

        let mut format = formats[0];
        for f in &formats {
            let fmt = f.format;
            if fmt == vk::Format::B8G8R8A8_UNORM || fmt == vk::Format::R8G8B8A8_UNORM {
                format = *f;
                break;
            }
        }

        let extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: desc.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: desc.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };

        let mut image_count = desc.image_count;
        if capabilities.max_image_count > 0 {
            image_count = image_count.min(capabilities.max_image_count);
        }
        image_count = image_count.max(capabilities.min_image_count);

        let loader = ash::khr::swapchain::Device::new(&self.0.instance.raw, &self.0.raw);

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.raw)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_array_layers(1)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .present_mode(vk::PresentModeKHR::MAILBOX)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let raw = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .map_err(|_| crate::Error::Unknown)?
        };

        let images = unsafe {
            loader
                .get_swapchain_images(raw)
                .map_err(|_| crate::Error::Unknown)?
        };

        let image_views = images
            .iter()
            .map(|&image| {
                let info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format.format)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .level_count(1)
                            .layer_count(1),
                    );

                unsafe {
                    self.0
                        .raw
                        .create_image_view(&info, None)
                        .map_err(|_| crate::Error::Unknown)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        println!(
            "Creating new VK swapchain with {:?}, {:?}, num images {}",
            format.format, format.color_space, image_count
        );

        Ok(Swapchain {
            raw,
            loader,
            image_views,
            device: Arc::clone(&self.0),
        })
    }
}

impl Drop for DeviceInner {
    fn drop(&mut self) {
        unsafe {
            self.raw.destroy_device(None);
        }
    }
}
