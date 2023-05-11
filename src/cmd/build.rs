use std::{env, path::PathBuf, process::Command};

use anyhow::{bail, Result};
use clap::Args;
use fluent_templates::Loader;
use fs_extra::dir::CopyOptions;
use mdbook::MDBook;
use novel_api::Timing;
use rayon::prelude::*;
use tracing::{error, info, warn};
use walkdir::WalkDir;

use crate::{
    utils::{self, CurrentDir},
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
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
    let mut timing = Timing::new();

    if utils::is_mdbook_dir(&config.build_path)? {
        execute_mdbook(config)?;
    } else {
        execute_pandoc(config)?;
    }
    println!("{}", utils::locales("build_complete_msg", "👌"));

    info!("Time spent on `build`: {}", timing.elapsed()?);

    Ok(())
}

pub fn execute_pandoc(config: Build) -> Result<()> {
    utils::ensure_executable_exists("pandoc")?;

    let input_markdown_file_path;
    let markdown_file_parent_path;
    let mut in_directory = false;

    if utils::is_markdown_file(&config.build_path)? {
        input_markdown_file_path = dunce::canonicalize(&config.build_path)?;
        markdown_file_parent_path = input_markdown_file_path.parent().unwrap().to_path_buf();
    } else if utils::is_markdown_dir(&config.build_path)? {
        let markdown_dir_path = dunce::canonicalize(&config.build_path)?;
        input_markdown_file_path = markdown_dir_path
            .join(markdown_dir_path.file_stem().unwrap())
            .with_extension("md");
        markdown_file_parent_path = markdown_dir_path;
        in_directory = true;
    } else {
        bail!("Invalid input path: `{}`", config.build_path.display());
    }
    info!(
        "Input markdown file path: `{}`",
        input_markdown_file_path.display()
    );
    println!("{}", utils::locales_with_arg("build_msg", "📚", "pandoc"));

    let output_epub_file_path = env::current_dir()?.join(utils::read_markdown_to_epub_file_name(
        &input_markdown_file_path,
    )?);
    info!(
        "Output epub file path: `{}`",
        output_epub_file_path.display()
    );

    if output_epub_file_path.try_exists()? {
        warn!("The epub output file already exists and will be deleted");
        utils::remove_file_or_dir(&output_epub_file_path)?;
    }

    let current_dir = CurrentDir::new(&markdown_file_parent_path)?;
    // TODO Could not determine image size for cover.webp: could not determine image type
    let mut pandoc = Command::new("pandoc")
        .arg("--from=commonmark+yaml_metadata_block")
        .arg("--to=epub3")
        .arg("--split-level=2")
        .arg("--epub-title-page=false")
        .args(["-o", output_epub_file_path.to_str().unwrap()])
        .arg(&input_markdown_file_path)
        .spawn()?;
    let status = pandoc.wait()?;
    if !status.success() {
        bail!("`pandoc` failed to execute");
    }

    if config.delete {
        if in_directory {
            utils::remove_file_or_dir(markdown_file_parent_path)?;
        } else {
            let images = utils::read_markdown_to_images(&input_markdown_file_path)?;
            images.par_iter().for_each(|image| {
                if let Err(error) = utils::remove_file_or_dir(image) {
                    error!("File deletion failed: `{}`, {}", image.display(), error);
                }
            });

            utils::remove_file_or_dir(input_markdown_file_path)?;
        }
    }

    if config.open {
        opener::open(output_epub_file_path)?;
    }

    current_dir.restore()?;

    Ok(())
}

pub fn execute_mdbook(config: Build) -> Result<()> {
    println!("{}", utils::locales_with_arg("build_msg", "📚", "mdBook"));

    let input_mdbook_dir_path = dunce::canonicalize(&config.build_path)?;
    info!(
        "Input mdbook directory path: `{}`",
        input_mdbook_dir_path.display()
    );

    let book_path = input_mdbook_dir_path.join("book");

    if book_path.try_exists()? {
        warn!("The output directory already exists and will be deleted");
        utils::remove_file_or_dir(&book_path)?;
    }

    let mdbook = MDBook::load(&input_mdbook_dir_path)?;
    mdbook.build()?;

    if config.delete {
        for entry in WalkDir::new(&input_mdbook_dir_path).max_depth(1) {
            let path = entry?.path().to_path_buf();

            if path != input_mdbook_dir_path && path != book_path {
                utils::remove_file_or_dir(path)?;
            }
        }

        let mut options = CopyOptions::new();
        options.copy_inside = true;
        options.content_only = true;
        fs_extra::dir::move_dir(&book_path, &input_mdbook_dir_path, &options)?;
    }

    if config.open {
        let path = if config.delete {
            input_mdbook_dir_path.join("index.html")
        } else {
            book_path.join("index.html")
        };

        opener::open_browser(path)?;
    }

    Ok(())
}
