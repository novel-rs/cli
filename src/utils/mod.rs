mod convert;
mod login;
mod markdown;
mod novel;
mod novel_info;
mod progress;
mod unicode;
mod writer;

pub(crate) use convert::*;
pub(crate) use login::*;
pub(crate) use markdown::*;
pub(crate) use novel::*;
pub(crate) use novel_info::*;
pub(crate) use progress::*;
pub(crate) use unicode::*;
pub(crate) use writer::*;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Result};
use image::{ColorType, DynamicImage};
use tracing::warn;

use crate::cmd::Convert;

pub(crate) fn file_stem<T>(path: T) -> Result<PathBuf>
where
    T: AsRef<Path>,
{
    let file_stem = PathBuf::from(
        path.as_ref()
            .file_stem()
            .ok_or_else(|| anyhow!("Can not get file_stem"))?,
    );

    Ok(file_stem)
}

#[must_use]
pub(crate) fn is_markdown<T>(path: T) -> bool
where
    T: AsRef<Path>,
{
    let path = path.as_ref();
    path.is_file() && novel_api::is_some_and(path.extension(), |ext| ext == "md")
}

#[must_use]
pub(crate) fn num_to_str(num: u16) -> String {
    if num < 10 {
        format!("00{num}")
    } else if num < 100 {
        format!("0{num}")
    } else {
        num.to_string()
    }
}

pub(crate) fn remove_file_or_dir<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.exists() {
        bail!("The item does not exist: {}", path.display());
    }

    if path.is_file() {
        if let Err(error) = trash::delete(path) {
            warn!(
                "Failed to move file `{}` to trash, the file will be permanently deleted: {}",
                path.display(),
                error
            );
            fs::remove_file(path)?;
        }
    } else if path.is_dir() {
        if let Err(error) = trash::delete(path) {
            warn!(
                "Failed to move directory `{}` to trash, the directory will be permanently deleted: {}",
                path.display(),
                error
            );
            fs::remove_dir_all(path)?;
        }
    } else {
        bail!("The item is neither a file nor a folder");
    }

    Ok(())
}

#[must_use]
pub(crate) fn lang(convert: &[Convert]) -> String {
    if convert.contains(&Convert::S2T) {
        String::from("zh-Hant")
    } else {
        String::from("zh-Hans")
    }
}

#[must_use]
pub(crate) fn image_ext(image: &DynamicImage) -> String {
    match image.color() {
        ColorType::Rgb8 | ColorType::Rgba8 => String::from("webp"),
        _ => String::from("png"),
    }
}
