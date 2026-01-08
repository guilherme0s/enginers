use std::sync::Arc;

use ash::vk;

use super::instance::InstanceInner;

pub struct Surface {
    pub(super) instance: Arc<InstanceInner>,
    pub(super) raw: vk::SurfaceKHR,
}

impl super::Instance {
    #[cfg(target_os = "linux")]
    pub(super) fn create_surface_from_wayland(
        &self,
        display: *mut vk::wl_display,
        surface: *mut vk::wl_surface,
    ) -> Result<vk::SurfaceKHR, crate::Error> {
        let loader = ash::khr::wayland_surface::Instance::new(&self.inner.entry, &self.inner.raw);
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
            self.instance.surface_loader.destroy_surface(self.raw, None);
        }
    }
}
