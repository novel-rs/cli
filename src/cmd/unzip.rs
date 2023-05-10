use std::{
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::Args;
use fluent_templates::Loader;
use novel_api::Timing;
use tracing::{info, warn};
use zip::ZipArchive;

use crate::{utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "unzip_command").unwrap())]
pub struct Unzip {
    #[arg(help = LOCALES.lookup(&LANG_ID, "epub_path").unwrap())]
    pub epub_path: PathBuf,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "delete").unwrap())]
    pub delete: bool,
}

pub fn execute(config: Unzip) -> Result<()> {
    let mut timing = Timing::new();

    utils::ensure_epub_file(&config.epub_path)?;

    unzip(&config.epub_path)?;

    if config.delete {
        utils::remove_file_or_dir(&config.epub_path)?;
    }

    info!("Time spent on `check`: {}", timing.elapsed()?);

    Ok(())
}

fn unzip<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    let output_dir = env::current_dir()?.join(path.file_stem().unwrap());
    if output_dir.try_exists()? {
        warn!("The output directory already exists and will be deleted");
        utils::remove_file_or_dir(&output_dir)?;
    }

    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        let outpath = output_dir.join(outpath);

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.try_exists()? {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}
