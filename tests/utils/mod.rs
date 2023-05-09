use std::{env, path::PathBuf};

use anyhow::Result;

#[allow(dead_code)]
pub fn test_data_path() -> Result<PathBuf> {
    Ok(PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("tests")
        .join("data"))
}
