use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use prost_reflect::{DescriptorPool, FileDescriptor};

pub const ERRORS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/errors.bin"));

pub fn load_file(path: &Path, includes: &[PathBuf]) -> Result<FileDescriptor> {
    let mut pool = load_pool(path, includes)?;
    Ok(add_error_definitions(&mut pool))
}

fn add_error_definitions(pool: &mut DescriptorPool) -> FileDescriptor {
    let primary_file = pool.files().last().unwrap().name().to_owned();

    if let Err(err) = pool.decode_file_descriptor_set(ERRORS) {
        tracing::warn!("failed to add additional protos to pool: {:#}", err);
    }

    pool.get_file_by_name(&primary_file).unwrap()
}

fn load_pool(path: &Path, includes: &[PathBuf]) -> Result<DescriptorPool> {
    tracing::info!(
        "add file with path '{}' and includes {:?}",
        path.display(),
        includes
    );

    match compile_proto(path, includes) {
        Ok(pool) => Ok(pool),
        Err(err) if err.is_parse() => match compile_file_set(path) {
            Ok(pool) => Ok(pool),
            Err(_) => {
                if path.extension() == Some(OsStr::new("proto")) {
                    bail!("{:?}", err)
                } else {
                    bail!("failed to parse file as either a protobuf source file or encoded file descriptor set: {}", err)
                }
            }
        },
        Err(err) => bail!("{:?}", err),
    }
}

fn compile_proto(path: &Path, includes: &[PathBuf]) -> Result<DescriptorPool, protox::Error> {
    let tmp;
    let includes = if includes.is_empty() {
        let Some(include) = path.parent() else {
            return Err(protox::Error::new("invalid path"));
        };

        tmp = [include.to_owned()];
        tmp.as_slice()
    } else {
        includes
    };

    Ok(protox::Compiler::new(includes)?
        .include_imports(true)
        .include_source_info(false)
        .open_file(path)?
        .descriptor_pool())
}

fn compile_file_set(path: &Path) -> Result<DescriptorPool> {
    let bytes = fs_err::read(path)?;
    DescriptorPool::decode(bytes.as_slice()).map_err(|err| anyhow::anyhow!("{:?}", err))
}
