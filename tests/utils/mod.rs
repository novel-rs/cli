use std::{env, path::PathBuf};

use anyhow::Result;

pub(crate) fn test_data_path() -> Result<PathBuf> {
    Ok(PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("tests")
        .join("data"))
}
