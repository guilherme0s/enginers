use rhi::{SwapchainDescriptor, vulkan::Instance};
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() -> Result<(), rhi::Error> {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .with_title("Triangle Example")
        .build(&event_loop)
        .unwrap();

    let instance = Instance::new()?;

    let surface = instance.create_surface_from_window(&window)?;
    let device = instance.create_device()?;

    let _ = device.create_swapchain(
        &surface,
        &SwapchainDescriptor {
            width: 800,
            height: 600,
            image_count: 3,
        },
    );

    Ok(())
}
