use std::{
    env,
    fs::{self, File},
    io::{BufReader, Read},
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

#[allow(dead_code)]
pub fn same_file_content<T, E>(lhs: T, rhs: E) -> bool
where
    T: AsRef<Path>,
    E: AsRef<Path>,
{
    if let (Ok(file_lhs), Ok(file_rhs)) = (File::open(lhs), File::open(rhs)) {
        let mut reader_lhs = BufReader::new(file_lhs);
        let mut reader_rhs = BufReader::new(file_rhs);
        let mut buf_lhs = [0; 256];
        let mut buf_rhs = [0; 256];

        while let (Ok(n_lhs), Ok(n_rhs)) =
            (reader_lhs.read(&mut buf_lhs), reader_rhs.read(&mut buf_rhs))
        {
            if n_lhs != n_rhs {
                return false;
            }
            if n_lhs == 0 {
                return true;
            }
            if buf_lhs != buf_rhs {
                return false;
            }
        }
    }

    false
}

pub fn test_data_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("CARGO_MANIFEST_DIR") {
        Ok(PathBuf::from(path).join("tests").join("data"))
    } else {
        Ok(env::current_dir()?.join("tests").join("data"))
    }
}
