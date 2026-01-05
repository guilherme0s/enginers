use rhi::vulkan::Instance;

fn main() -> Result<(), rhi::Error> {
    let _ = Instance::new()?;

    Ok(())
}
