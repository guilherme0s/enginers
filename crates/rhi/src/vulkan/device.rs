pub struct Device {
    pub(super) raw: ash::Device,
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.raw.destroy_device(None);
        }
    }
}
