use ash::vk;

pub struct Swapchain {
    pub(super) raw: vk::SwapchainKHR,
    pub(super) loader: ash::khr::swapchain::Device,
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_swapchain(self.raw, None);
        }
    }
}
