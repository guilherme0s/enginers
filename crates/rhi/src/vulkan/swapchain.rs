use std::sync::Arc;

use ash::vk;

pub struct Swapchain {
    pub(super) device: Arc<super::DeviceInner>,
    pub(super) raw: vk::SwapchainKHR,
    pub(super) loader: ash::khr::swapchain::Device,
    pub(super) image_views: Vec<vk::ImageView>,
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for &view in &self.image_views {
                self.device.raw.destroy_image_view(view, None);
            }

            self.loader.destroy_swapchain(self.raw, None);
        }
    }
}
