use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> anyhow::Result<()> {
    vergen::EmitBuilder::builder().git_sha(true).emit()?;

    compile_protos(
        ["googleapis", "proto"],
        [
            "google/rpc/status.proto",
            "google/rpc/error_details.proto",
            "lanquetta/errors.proto",
        ],
        "errors.bin",
    );

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

fn compile_protos(
    includes: impl IntoIterator<Item = impl AsRef<Path>>,
    files: impl IntoIterator<Item = impl AsRef<Path>>,
    destination: impl AsRef<Path>,
) {
    let mut compiler = protox::Compiler::new(includes).unwrap();
    compiler
        .include_source_info(false)
        .include_imports(true)
        .open_files(files)
        .unwrap();

    for file in compiler.files() {
        if let Some(path) = file.path() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join(destination),
        compiler.encode_file_descriptor_set(),
    )
    .unwrap();
}
