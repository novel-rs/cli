use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use fs_extra::dir::CopyOptions;

#[allow(dead_code)]
pub fn copy_to_temp_dir<T, F>(from: T, to: F) -> Result<PathBuf>
where
    T: AsRef<Path>,
    F: AsRef<Path>,
{
    let target = to.as_ref().join(&from);
    let from = test_data_path()?.join(from);

    if from.is_dir() {
        let mut options = CopyOptions::new();
        options.copy_inside = true;
        fs_extra::dir::copy(from, to, &options)?;

        if !target.is_dir() {
            bail!("Copy failed: `{}`", target.display())
        }
    } else if from.is_file() {
        fs::copy(from, &target)?;

        if !target.is_file() {
            bail!("Copy failed: `{}`", target.display())
        }
    } else {
        bail!("`{}` is neither a file nor a directory", from.display());
    }

    Ok(target)
}

fn test_data_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("CARGO_MANIFEST_DIR") {
        Ok(PathBuf::from(path).join("tests").join("data"))
    } else {
        Ok(env::current_dir()?.join("tests").join("data"))
    }
}
