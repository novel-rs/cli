use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Result};
use clap::Args;
use fluent_templates::Loader;
use tracing::warn;
use zip::ZipArchive;

use crate::{utils, LANG_ID, LOCALES};

#[derive(Debug, Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "unzip_command").expect("`unzip_command` does not exists"))]
pub struct Unzip {
    #[arg(help = LOCALES.lookup(&LANG_ID, "epub_path").expect("`epub_path` does not exists"))]
    pub epub_path: PathBuf,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "delete").expect("`delete` does not exists"))]
    pub delete: bool,
}

pub fn execute(config: Unzip) -> Result<()> {
    ensure_epub(&config.epub_path)?;

    unzip(&config.epub_path)?;

    if config.delete {
        utils::remove_file_or_dir(&config.epub_path)?;
    }

    Ok(())
}

fn ensure_epub<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    ensure!(path.exists(), "File `{}` does not exist", path.display());
    ensure!(path.is_file(), "`{}` is not file", path.display());
    ensure!(
        novel_api::is_some_and(path.extension(), |extension| extension == "epub"),
        "File `{}` is not epub file",
        path.display()
    );

    Ok(())
}

fn unzip<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    let output_directory = utils::file_stem(path)?;
    if output_directory.exists() {
        warn!("The output directory already exists and will be deleted");
        utils::remove_file_or_dir(&output_directory)?;
    }

    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        let outpath = output_directory.join(outpath);

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
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
