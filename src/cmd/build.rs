use std::{fs, path::PathBuf, process::Command};

use anyhow::{bail, ensure, Result};
use clap::Args;
use fluent_templates::Loader;
use fs_extra::dir::CopyOptions;
use mdbook::MDBook;
use novel_api::Timing;
use tracing::{info, warn};
use walkdir::WalkDir;

use crate::{
    utils::{self, CurrentDir},
    LANG_ID, LOCALES,
};

use super::Format;

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "build_command").unwrap())]
pub struct Build {
    #[arg(help = LOCALES.lookup(&LANG_ID, "build_path").unwrap())]
    pub build_path: PathBuf,

    #[arg(short, long, value_enum,
        help = LOCALES.lookup(&LANG_ID, "format").unwrap())]
    pub format: Format,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "delete").unwrap())]
    pub delete: bool,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "open").unwrap())]
    pub open: bool,
}

pub fn execute(config: Build) -> Result<()> {
    ensure!(
        config.build_path.is_dir(),
        "The build input directory does not exist: `{}`",
        config.build_path.display()
    );

    match config.format {
        Format::Pandoc => {
            println!("{}", utils::locales_with_arg("build_msg", "📚", "pandoc"));

            let mut timing = Timing::new();
            execute_pandoc(config)?;
            info!("Time spent on `pandoc build`: {}", timing.elapsed()?);
        }
        Format::Mdbook => {
            println!("{}", utils::locales_with_arg("build_msg", "📚", "mdBook"));

            let mut timing = Timing::new();
            execute_mdbook(config)?;
            info!("Time spent on `mdbook build`: {}", timing.elapsed()?);
        }
    }

    println!("{}", utils::locales("build_complete_msg", "✔️"));

    Ok(())
}

pub fn execute_pandoc(config: Build) -> Result<()> {
    let dir_name = PathBuf::from(config.build_path.file_name().unwrap());
    let md_file_name = dir_name.with_extension("md");
    let epub_file_name = dir_name.with_extension("epub");

    let input_path = fs::canonicalize(&config.build_path)?.join(md_file_name);
    let output_path = fs::canonicalize(&config.build_path)?
        .parent()
        .unwrap()
        .join(epub_file_name);

    if output_path.try_exists()? {
        warn!("The epub output file already exists and will be deleted");
        utils::remove_file_or_dir(&output_path)?;
    }

    let current_dir = CurrentDir::new(&config.build_path)?;
    // TODO Could not determine image size for cover.webp: could not determine image type
    let mut pandoc = Command::new("pandoc")
        .arg("--from=commonmark+yaml_metadata_block")
        .arg("--to=epub3")
        .arg("--split-level=2")
        .arg("--epub-title-page=false")
        .args(["-o", output_path.to_str().unwrap()])
        .arg(input_path)
        .spawn()?;
    let status = pandoc.wait()?;
    if !status.success() {
        bail!("`pandoc` failed to execute");
    }
    current_dir.restore()?;

    if config.delete {
        utils::remove_file_or_dir(&config.build_path)?;
    }

    if config.open {
        info!("Open file: `{}`", output_path.display());
        opener::open(output_path)?;
    }

    Ok(())
}

pub fn execute_mdbook(config: Build) -> Result<()> {
    let book_path = config.build_path.join("book");

    if book_path.try_exists()? {
        warn!("The output directory already exists and will be deleted");
        utils::remove_file_or_dir(&book_path)?;
    }

    let mdbook = MDBook::load(&config.build_path)?;
    mdbook.build()?;

    if config.delete {
        for entry in WalkDir::new(&config.build_path).max_depth(1) {
            let path = entry?.path().to_path_buf();

            if path != config.build_path && path != book_path {
                utils::remove_file_or_dir(path)?;
            }
        }

        let mut options = CopyOptions::new();
        options.copy_inside = true;
        options.content_only = true;
        fs_extra::dir::move_dir(&book_path, &config.build_path, &options)?;
    }

    if config.open {
        let path = if config.delete {
            config.build_path.join("index.html")
        } else {
            book_path.join("index.html")
        };

        info!("Open file: `{}`", path.display());
        opener::open_browser(path)?;
    }

    Ok(())
}
