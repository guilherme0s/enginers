pub mod vulkan;

#[derive(Debug)]
pub enum Error {
    Unknown,
}

pub struct RHIRenderPassDescriptor {}

pub trait RHIDevice {
    type CommandEncoder: RHICommandEncoder;

    fn create_command_encoder(&self) -> Result<Self::CommandEncoder, Error>;
}

pub trait RHICommandEncoder {
    type CommandBuffer;

    /// The backend-specific render pass encoder type.
    ///
    /// The lifetime `'a` ensures the render pass cannot outlive the encoder,
    /// enforcing proper nesting and preventing dangling references.
    type CommandEncoderRenderPass<'a>: RHICommandEncoderRenderPass
    where
        Self: 'a;

    fn begin_render_pass<'a>(
        &'a mut self,
        desc: &RHIRenderPassDescriptor,
    ) -> Result<Self::CommandEncoderRenderPass<'a>, Error>;

    fn finish(self) -> Result<Self::CommandBuffer, Error>;
}

pub trait RHICommandEncoderRenderPass {}
