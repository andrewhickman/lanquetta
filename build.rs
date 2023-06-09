fn main() -> anyhow::Result<()> {
    vergen::EmitBuilder::builder().git_sha(true).emit()?;

    #[cfg(windows)]
    winres::WindowsResource::new().set_icon("img/logo.ico").compile()?;

    Ok(())
}
