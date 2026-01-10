use ash::vk;

use crate::{RHICommandEncoder, RHICommandEncoderRenderPass};

pub struct CommandEncoder {
    pub(super) device: ash::Device,
    pub(super) cmd_pool: vk::CommandPool,
    pub(super) cmd_buffer: vk::CommandBuffer,
}

pub struct CommandEncoderRenderPass<'a> {
    #[allow(unused)]
    encoder: &'a mut CommandEncoder,
}

impl RHICommandEncoder for CommandEncoder {
    type CommandBuffer = vk::CommandBuffer;

    type CommandEncoderRenderPass<'a>
        = CommandEncoderRenderPass<'a>
    where
        Self: 'a;

    fn begin_render_pass<'a>(
        &'a mut self,
        #[allow(unused)] desc: &crate::RHIRenderPassDescriptor,
    ) -> Result<Self::CommandEncoderRenderPass<'a>, crate::Error> {
        // TODO

        Ok(CommandEncoderRenderPass { encoder: self })
    }

    fn finish(self) -> Result<Self::CommandBuffer, crate::Error> {
        unsafe {
            self.device
                .end_command_buffer(self.cmd_buffer)
                .map_err(|_| crate::Error::Unknown)?
        };

        Ok(self.cmd_buffer)
    }
}

impl RHICommandEncoderRenderPass for CommandEncoderRenderPass<'_> {}

impl Drop for CommandEncoder {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.cmd_pool, None);
        }
    }
}
