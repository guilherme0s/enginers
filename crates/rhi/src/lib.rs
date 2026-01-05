pub mod vulkan;

#[derive(Debug)]
pub enum Error {
    Unknown,
}

pub struct SwapchainDescriptor {
    pub width: u32,
    pub height: u32,
    pub image_count: u32,
}
