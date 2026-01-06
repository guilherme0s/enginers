use rhi::vulkan::instance::Instance;

fn main() -> Result<(), rhi::Error> {
    let _ = Instance::new()?;

    Ok(())
}
