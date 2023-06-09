use winres::WindowsResource;

fn main() -> anyhow::Result<()> {
    vergen::EmitBuilder::builder().git_sha(true).emit()?;

    #[cfg(windows)]
    WindowsResource::new().set_icon("img/icon.ico").compile()?;

    Ok(())
}
