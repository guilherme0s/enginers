use ash::vk;

pub struct Surface {
    pub(super) raw: vk::SurfaceKHR,
    pub(super) loader: ash::khr::surface::Instance,
}

impl super::Instance {
    #[cfg(target_os = "linux")]
    pub(super) fn create_surface_from_wayland(
        &self,
        display: *mut vk::wl_display,
        surface: *mut vk::wl_surface,
    ) -> Result<vk::SurfaceKHR, crate::Error> {
        let loader = ash::khr::wayland_surface::Instance::new(&self.entry, &self.raw);
        let create_info = vk::WaylandSurfaceCreateInfoKHR::default()
            .flags(vk::WaylandSurfaceCreateFlagsKHR::empty())
            .display(display)
            .surface(surface);

        let raw = unsafe {
            loader
                .create_wayland_surface(&create_info, None)
                .map_err(|_| crate::Error::Unknown)?
        };

        Ok(raw)
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.raw, None);
        }
    }
}
