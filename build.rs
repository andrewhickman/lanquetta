use std::{env, fs, path::PathBuf};

fn main() -> anyhow::Result<()> {
    vergen::EmitBuilder::builder().git_sha(true).emit()?;

    let mut compiler = protox::Compiler::new(["grpc/src/proto", "googleapis", "proto"]).unwrap();
    compiler
        .include_source_info(false)
        .include_imports(true)
        .open_files([
            "grpc/health/v1/health.proto",
            "grpc/reflection/v1/reflection.proto",
            "google/rpc/status.proto",
            "google/rpc/error_details.proto",
            "lanquetta.proto",
        ])
        .unwrap();

    for file in compiler.files() {
        if let Some(path) = file.path() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("proto.bin"),
        compiler.encode_file_descriptor_set(),
    )
    .unwrap();

    #[cfg(windows)]
    winres::WindowsResource::new()
        .set_icon_with_id(
            "img/logo.ico",
            &(windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION.0 as u32).to_string(),
        )
        .compile()?;
    println!("cargo:rerun-if-changed=img/logo.ico");

    Ok(())
}
