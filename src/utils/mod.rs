mod check;
mod concurrency;
mod convert;
mod current_dir;
mod image;
mod login;
mod markdown;
mod novel;
mod novel_info;
mod progress;
mod unicode;
mod writer;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

pub use check::*;
use color_eyre::eyre::{bail, Result};
pub use concurrency::*;
use console::{Alignment, Emoji};
pub use convert::*;
pub use current_dir::*;
use fluent_templates::Loader;
pub use login::*;
pub use markdown::*;
pub use novel::*;
pub use novel_info::*;
pub use progress::*;
use sanitize_filename::Options;
use tracing::{error, info, warn};
pub use unicode::*;
pub use writer::*;

pub use self::image::*;
use crate::{cmd::Convert, LANG_ID, LOCALES};

#[inline]
#[must_use]
pub fn num_to_str(num: u16) -> String {
    if num < 10 {
        format!("00{num}")
    } else if num < 100 {
        format!("0{num}")
    } else {
        num.to_string()
    }
}

pub fn remove_file_or_dir<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    remove_file_or_dir_all(&[path])
}

pub fn remove_file_or_dir_all<T>(paths: &[T]) -> Result<()>
where
    T: AsRef<Path>,
{
    for path in paths {
        let path = path.as_ref();

        if !path.try_exists()? {
            bail!("The item does not exist: `{}`", path.display());
        }

        let path = dunce::canonicalize(path)?;
        if path.is_file() {
            info!("File `{}` will be deleted", path.display());
        } else if path.is_dir() {
            info!("Directory `{}` will be deleted", path.display());
        } else {
            bail!("The item is neither a file nor a folder");
        }
    }

    if let Err(error) = trash::delete_all(paths) {
        error!("Failed to put file or folder into Trash: {}", error);
    }

    for path in paths {
        let path = path.as_ref();

        if path.try_exists()? {
            error!(
                "Failed to put file or folder into Trash: {}",
                path.display()
            );

            if path.is_file() {
                fs::remove_file(path)?;
            } else {
                fs::remove_dir_all(path)?;
            }
        }
    }

    Ok(())
}

pub fn lang<T>(convert: T) -> Lang
where
    T: AsRef<[Convert]>,
{
    if convert.as_ref().contains(&Convert::S2T) {
        Lang::ZhHant
    } else {
        Lang::ZhHans
    }
}

#[must_use]
pub fn emoji<T>(str: T) -> String
where
    T: AsRef<str>,
{
    let emoji = Emoji(str.as_ref(), ">").to_string();
    console::pad_str(&emoji, 2, Alignment::Left, None).to_string()
}

#[must_use]
pub fn locales<T, E>(name: T, emoji: E) -> String
where
    T: AsRef<str>,
    E: AsRef<str>,
{
    let args = {
        let mut map = HashMap::new();
        map.insert(String::from("emoji"), self::emoji(emoji).into());
        map
    };

    LOCALES.lookup_with_args(&LANG_ID, name.as_ref(), &args)
}

#[must_use]
pub fn locales_with_arg<T, E, F>(name: T, emoji: E, arg: F) -> String
where
    T: AsRef<str>,
    E: AsRef<str>,
    F: AsRef<str>,
{
    let args = {
        let mut map = HashMap::new();
        map.insert(String::from("emoji"), self::emoji(emoji).into());
        map.insert(String::from("arg"), arg.as_ref().into());
        map
    };

    LOCALES.lookup_with_args(&LANG_ID, name.as_ref(), &args)
}

#[must_use]
pub fn to_novel_dir_name<T>(novel_name: T) -> PathBuf
where
    T: AsRef<str>,
{
    if !sanitize_filename::is_sanitized(&novel_name) {
        warn!("The output file name is invalid and has been modified");
    }

    do_sanitize(novel_name)
}

#[must_use]
pub fn to_markdown_file_name<T>(novel_name: T) -> PathBuf
where
    T: AsRef<str>,
{
    do_sanitize(novel_name).with_extension("md")
}

#[must_use]
pub fn to_epub_file_name<T>(novel_name: T) -> PathBuf
where
    T: AsRef<str>,
{
    do_sanitize(novel_name).with_extension("epub")
}

fn do_sanitize<T>(novel_name: T) -> PathBuf
where
    T: AsRef<str>,
{
    let option = Options {
        replacement: " ",
        ..Default::default()
    };

    PathBuf::from(sanitize_filename::sanitize_with_options(novel_name, option))
}

pub fn read_markdown_to_epub_file_name<T>(markdown_path: T) -> Result<PathBuf>
where
    T: AsRef<Path>,
{
    let metadata = get_metadata_from_file(markdown_path)?;
    Ok(to_epub_file_name(metadata.title))
}
