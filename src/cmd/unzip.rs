use std::{
    fmt::Write,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Result};
use clap::Args;
use fluent_templates::Loader;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use ripunzip::{UnzipEngine, UnzipOptions, UnzipProgressReporter};
use tracing::warn;

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

    let file = File::open(&config.epub_path)?;
    let output_directory = utils::file_stem(&config.epub_path)?;

    if output_directory.exists() {
        warn!("The output directory already exists and will be deleted");
        utils::remove_file_or_dir(&output_directory)?;
    }

    let options = UnzipOptions {
        output_directory: Some(output_directory),
        single_threaded: false,
    };

    let engine = UnzipEngine::for_file(file, options, ProgressDisplayer::new())?;
    engine.unzip()?;

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

struct ProgressDisplayer {
    pb: ProgressBar,
}

impl ProgressDisplayer {
    fn new() -> Self {
        Self {
            pb: ProgressBar::new(0),
        }
    }
}

impl UnzipProgressReporter for ProgressDisplayer {
    fn extraction_starting(&self, display_name: &str) {
        self.pb.set_message(format!("Extracting {}", display_name))
    }

    fn total_bytes_expected(&self, expected: u64) {
        self.pb.set_length(expected);
        self.pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})\n{msg}")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#-"));
    }

    fn bytes_extracted(&self, count: u64) {
        self.pb.inc(count)
    }
}
