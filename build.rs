fn main() -> anyhow::Result<()> {
    vergen::EmitBuilder::builder().git_sha(true).emit()?;

    #[cfg(windows)]
    winres::WindowsResource::new()
        .set_icon_with_id(
            "img/logo.ico",
            &(windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION.0 as u32).to_string(),
        )
        .compile()?;

    Ok(())
}
