use std::{path::PathBuf, process::Command};

use anyhow::{bail, ensure, Result};
use clap::Args;
use fluent_templates::Loader;
use fs_extra::dir::CopyOptions;
use mdbook::MDBook;
use novel_api::Timing;
use tracing::{info, warn};

use crate::{utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Debug, Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "build_command").unwrap())]
pub struct Build {
    #[arg(help = LOCALES.lookup(&LANG_ID, "build_path").unwrap())]
    pub build_path: PathBuf,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "delete").unwrap())]
    pub delete: bool,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "open").unwrap())]
    pub open: bool,
}

pub fn execute(config: Build) -> Result<()> {
    ensure!(config.build_path.exists(), "Build input does not exist");

    let mut timing = Timing::new();

    if utils::is_markdown(&config.build_path) {
        println!("{}", utils::locales_with_arg("build_msg", "📚", "pandoc"));
        execute_pandoc(config)?;
        info!("Time spent on `pandoc build`: {}", timing.elapsed()?);
    } else if config.build_path.is_dir() {
        println!("{}", utils::locales_with_arg("build_msg", "📚", "mdBook"));
        execute_mdbook(config)?;
        info!("Time spent on `mdBook build`: {}", timing.elapsed()?);
    } else {
        bail!("Unsupported input format")
    }

    println!("{}", utils::locales("build_complete_msg", "✔️"));

    Ok(())
}

pub fn execute_pandoc(config: Build) -> Result<()> {
    let mut output_path = utils::file_stem(&config.build_path)?;
    output_path.set_extension("epub");
    if output_path.exists() {
        warn!("The epub output file already exists and will be overwritten");
    }

    let mut pandoc = Command::new("pandoc")
        .arg("--split-level=2")
        .arg("--epub-title-page=false")
        .args(["-o", output_path.to_str().unwrap()])
        .arg(&config.build_path)
        .spawn()?;
    let status = pandoc.wait()?;
    if !status.success() {
        bail!("pandoc run failed");
    }

    if config.delete {
        utils::remove_file_or_dir(&config.build_path)?;

        let images_path = config.build_path.parent().unwrap().join("images");
        if images_path.exists() {
            utils::remove_file_or_dir(&images_path)?;
        }
    }

    if config.open {
        opener::open(output_path)?;
    }

    Ok(())
}

pub fn execute_mdbook(config: Build) -> Result<()> {
    let book_path = config.build_path.join("book");

    if book_path.exists() {
        warn!("The output directory already exists and will be deleted");
        utils::remove_file_or_dir(&book_path)?;
    }

    let mdbook = MDBook::load(&config.build_path)?;
    mdbook.build()?;

    if config.delete {
        let mut options = CopyOptions::new();
        options.copy_inside = true;
        options.content_only = true;
        fs_extra::dir::copy(&book_path, &config.build_path, &options)?;

        utils::remove_file_or_dir(&book_path)?;
    }

    if config.open {
        if config.delete {
            opener::open_browser(config.build_path.join("index.html"))?;
        } else {
            opener::open_browser(book_path.join("index.html"))?;
        }
    }

    Ok(())
}
