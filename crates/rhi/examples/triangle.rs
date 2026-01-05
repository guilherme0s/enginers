use rhi::vulkan::Instance;

fn main() -> Result<(), rhi::Error> {
    let instance = Instance::new()?;
    let _ = instance.create_device()?;

    Ok(())
}
