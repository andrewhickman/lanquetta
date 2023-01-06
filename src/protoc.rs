use std::path::Path;

use anyhow::{bail, Result};
use prost_reflect::DescriptorPool;

pub fn load_file(path: &Path) -> Result<DescriptorPool> {
    match path.parent() {
        Some(include) => Ok(DescriptorPool::decode(
            protox::Compiler::new([include])?
                .include_imports(true)
                .include_source_info(false)
                .open_file(path)?
                .encode_file_descriptor_set()
                .as_slice(),
        )?),
        None => bail!("invalid path"),
    }
}
