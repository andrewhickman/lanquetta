use std::{ffi::OsStr, path::Path};

use anyhow::{bail, Result};
use prost_reflect::DescriptorPool;

pub fn load_file(path: &Path) -> Result<DescriptorPool> {
    match compile(path) {
        Ok(pool) => Ok(pool),
        Err(err) if err.is_parse() => match load(path) {
            Ok(pool) => Ok(pool),
            Err(_) => {
                if path.extension() == Some(OsStr::new("proto")) {
                    bail!("{:?}", err)
                } else {
                    bail!("failed to parse '{}' as either a protobuf source file or encoded file descriptor set: {}", path.display(), err)
                }
            }
        },
        Err(err) => bail!("{:?}", err),
    }
}

fn compile(path: &Path) -> Result<DescriptorPool, protox::Error> {
    match path.parent() {
        Some(include) => Ok(protox::Compiler::new([include])?
            .include_imports(true)
            .include_source_info(false)
            .open_file(path)?
            .descriptor_pool()),
        None => Err(protox::Error::new("invalid path")),
    }
}

fn load(path: &Path) -> Result<DescriptorPool> {
    let bytes = fs_err::read(path)?;
    DescriptorPool::decode(bytes.as_slice()).map_err(|err| anyhow::anyhow!("{:?}", err))
}
