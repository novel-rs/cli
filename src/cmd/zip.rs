use std::{
    fs::File,
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};

use anyhow::{ensure, Result};
use clap::{value_parser, Args};
use fluent_templates::Loader;
use tracing::warn;
use walkdir::{DirEntry, WalkDir};
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

use crate::{utils, LANG_ID, LOCALES};

#[derive(Debug, Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "zip_command").expect("`zip_command` does not exists"))]
pub struct Zip {
    #[arg(help = LOCALES.lookup(&LANG_ID, "epub_dir_path").expect("`epub_dir_path` does not exists"))]
    pub epub_dir_path: PathBuf,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "delete").expect("`delete` does not exists"))]
    pub delete: bool,

    #[arg(long, default_value_t = 6,
        value_parser = value_parser!(i32).range(0..=9),
        help = LOCALES.lookup(&LANG_ID, "level").expect("`level` does not exists"))]
    pub level: i32,
}

pub fn execute(config: Zip) -> Result<()> {
    ensure_epub_dir(&config.epub_dir_path)?;

    let mut epub_file_path = utils::file_stem(&config.epub_dir_path)?;
    epub_file_path.set_extension("epub");

    if epub_file_path.exists() {
        warn!("The epub output file already exists and will be overwritten");
    }

    let file = File::create(epub_file_path)?;

    let walkdir = WalkDir::new(&config.epub_dir_path);
    let it = walkdir.into_iter();

    zip_dir(
        &mut it.filter_map(|e| e.ok()),
        &config.epub_dir_path,
        file,
        config.level,
    )?;

    if config.delete {
        utils::remove_file_or_dir(&config.epub_dir_path)?;
    }

    Ok(())
}

fn ensure_epub_dir<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    ensure!(path.exists(), "Dir `{}` does not exist", path.display());
    ensure!(path.is_dir(), "`{}` is not dir", path.display());

    Ok(())
}

fn zip_dir<T, E>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: T,
    writer: E,
    level: i32,
) -> Result<()>
where
    T: AsRef<Path>,
    E: Write + Seek,
{
    let mut zip = ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(level));

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix.as_ref())?;

        if path.is_file() {
            zip.start_file(name.to_str().unwrap(), options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_str().unwrap(), options)?;
        }
    }
    zip.finish()?;

    Ok(())
}
