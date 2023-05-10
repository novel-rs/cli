use std::{
    env,
    fs::File,
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::Args;
use fluent_templates::Loader;
use novel_api::Timing;
use tracing::{info, warn};
use walkdir::{DirEntry, WalkDir};
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

use crate::{utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "zip_command").unwrap())]
pub struct Zip {
    #[arg(help = LOCALES.lookup(&LANG_ID, "epub_dir_path").unwrap())]
    pub epub_dir_path: PathBuf,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "delete").unwrap())]
    pub delete: bool,
}

pub fn execute(config: Zip) -> Result<()> {
    let mut timing = Timing::new();

    utils::ensure_epub_dir(&config.epub_dir_path)?;

    let epub_file_path = env::current_dir()?
        .join(config.epub_dir_path.file_stem().unwrap())
        .with_extension("epub");
    if epub_file_path.try_exists()? {
        warn!("The epub output file already exists and will be deleted");
        utils::remove_file_or_dir(&epub_file_path)?;
    }

    let file = File::create(epub_file_path)?;
    let walkdir = WalkDir::new(&config.epub_dir_path);
    zip_dir(
        &mut walkdir.into_iter().filter_map(|e| e.ok()),
        &config.epub_dir_path,
        file,
    )?;

    if config.delete {
        utils::remove_file_or_dir(&config.epub_dir_path)?;
    }

    info!("Time spent on `check`: {}", timing.elapsed()?);

    Ok(())
}

fn zip_dir<T, E>(iter: &mut dyn Iterator<Item = DirEntry>, prefix: T, writer: E) -> Result<()>
where
    T: AsRef<Path>,
    E: Write + Seek,
{
    let mut zip = ZipWriter::new(writer);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut buffer = Vec::new();
    for entry in iter {
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
