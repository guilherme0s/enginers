use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use rhi::vulkan::Instance;
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() -> Result<(), rhi::Error> {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .with_title("Triangle Example")
        .build(&event_loop)
        .unwrap();

    let instance = Instance::new()?;

    let surface = instance.create_surface(
        window.display_handle().unwrap().as_raw(),
        window.window_handle().unwrap().as_raw(),
    )?;

    let device = instance.create_device()?;
    let swapchain = device.swapchain_create(&surface, 800, 600)?;
    device.swapchain_destroy(swapchain);

    Ok(())
}
