use ash::vk;

pub struct Surface {
    pub(super) raw: vk::SurfaceKHR,
    pub(super) loader: ash::khr::surface::Instance,
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.raw, None);
        }
    }
}
