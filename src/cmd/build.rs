use std::{env, fs, path::PathBuf, process::Command};

use anyhow::{bail, ensure, Result};
use clap::Args;
use fluent_templates::Loader;
use fs_extra::dir::CopyOptions;
use mdbook::MDBook;
use novel_api::Timing;
use tracing::{info, warn};
use walkdir::WalkDir;

use crate::{utils, LANG_ID, LOCALES};

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
    ensure!(config.build_path.is_dir(), "Build input does not exist");

    let mut timing = Timing::new();

    if config.format == Format::Pandoc {
        println!("{}", utils::locales_with_arg("build_msg", "📚", "pandoc"));
        execute_pandoc(config)?;
        info!("Time spent on `pandoc build`: {}", timing.elapsed()?);
    } else if config.format == Format::Mdbook {
        println!("{}", utils::locales_with_arg("build_msg", "📚", "mdBook"));
        execute_mdbook(config)?;
        info!("Time spent on `mdBook build`: {}", timing.elapsed()?);
    } else {
        unreachable!(
            "Unsupported input format: `{}`",
            config.build_path.display()
        );
    }

    println!("{}", utils::locales("build_complete_msg", "✔️"));

    Ok(())
}

pub fn execute_pandoc(config: Build) -> Result<()> {
    let input_path = config.build_path.with_extension("md");
    let output_path = fs::canonicalize(&config.build_path)?
        .parent()
        .unwrap()
        .join(config.build_path.with_extension("epub"));

    if output_path.exists() {
        warn!("The epub output file already exists and will be deleted");
        utils::remove_file_or_dir(&output_path)?;
    }

    let backup = env::current_dir()?;
    env::set_current_dir(&config.build_path)?;

    let mut pandoc = Command::new("pandoc")
        .arg("--split-level=2")
        .arg("--epub-title-page=false")
        .args(["-o", output_path.to_str().unwrap()])
        .arg(input_path)
        .spawn()?;
    let status = pandoc.wait()?;
    if !status.success() {
        bail!("`pandoc` failed to execute");
    }

    env::set_current_dir(backup)?;

    if config.delete {
        utils::remove_file_or_dir(&config.build_path)?;
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
        if config.delete {
            opener::open_browser(config.build_path.join("index.html"))?;
        } else {
            opener::open_browser(book_path.join("index.html"))?;
        }
    }

    Ok(())
}
