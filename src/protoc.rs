use std::{
    env,
    fmt::Write,
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use prost_reflect::FileDescriptor;

pub fn load_file(path: &Path) -> Result<FileDescriptor> {
    match try_decode(path) {
        Ok(file) => return Ok(file),
        Err(err) => {
            tracing::debug!("failed to parse file as descriptor set: {:?}", err)
        }
    }

    let protoc = protoc_path()?;

    let mut cmd = Command::new(protoc);

    cmd.stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped());

    cmd.arg("-I")
        .arg(path.parent().context("invalid file path")?);

    for include in protoc_includes() {
        cmd.arg("-I").arg(include);
    }

    cmd.arg("--include_imports");

    let out_path = temp_path()?;
    cmd.arg("-o").arg(&out_path);

    cmd.arg(path);

    tracing::debug!("Running {:?}", cmd);
    let output = cmd.output().context("failed to run protoc")?;

    if !output.status.success() {
        if let Err(err) = fs_err::remove_file(out_path) {
            tracing::error!("failed to delete temporary file: {:?}", err);
        }
        bail!(
            "protoc returned {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        )
    } else {
        let bytes = fs_err::read(&out_path).context("failed to read protoc output")?;
        if let Err(err) = fs_err::remove_file(out_path) {
            tracing::error!("failed to delete temporary file: {:?}", err);
        }
        FileDescriptor::decode(bytes.as_ref()).context("failed to decode protobuf output")
    }
}

fn try_decode(path: &Path) -> Result<FileDescriptor> {
    let bytes = fs_err::read(path)?;
    Ok(FileDescriptor::decode(bytes.as_ref())?)
}

fn protoc_path() -> Result<PathBuf> {
    match env::var_os("PROTOC") {
        Some(protoc) => Ok(PathBuf::from(protoc)),
        None => which::which("protoc").context("protoc command not found in PATH"),
    }
}

fn protoc_includes() -> Vec<PathBuf> {
    match env::var_os("PROTOC_INCLUDE") {
        Some(includes) => env::split_paths(&includes).collect(),
        None => vec![],
    }
}

fn temp_path() -> Result<PathBuf> {
    const RETRY_COUNT: u32 = 10;

    let temp_dir = env::temp_dir();
    for _ in 0..RETRY_COUNT {
        let mut name = String::from("grpc-client-");
        let rand_bytes: [u8; 8] = rand::random();
        for byte in rand_bytes {
            write!(name, "{:x}", byte).unwrap();
        }
        name.push_str(".bin");

        let temp_file = temp_dir.join(name);

        match fs_err::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_file)
        {
            Ok(_) => return Ok(temp_file),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(anyhow::Error::from(err).context("failed to create temporary file"))
            }
        }
    }

    bail!("too many temporary files")
}
