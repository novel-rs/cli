use std::{fs, path::PathBuf};

use anyhow::{ensure, Result};
use clap::Args;
use fluent_templates::Loader;
use novel_api::Timing;
use pulldown_cmark::{Event, Options, Parser};
use rayon::prelude::*;
use tracing::info;

use crate::{cmd::Convert, utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "transform_command").unwrap())]
pub struct Transform {
    #[arg(help = LOCALES.lookup(&LANG_ID, "markdown_path").unwrap())]
    pub markdown_path: PathBuf,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").unwrap())]
    pub converts: Vec<Convert>,
}

pub fn execute(config: Transform) -> Result<()> {
    let mut timing = Timing::new();

    let (mut meta_data, markdown) = utils::read_markdown(&config.markdown_path)?;

    ensure!(
        meta_data.lang_is_ok(),
        "The lang field must be zh-Hant or zh-Hans: {}",
        meta_data.lang
    );
    ensure!(
        meta_data.cover_image_is_ok(),
        "Cover image does not exist: {}",
        meta_data.cover_image.unwrap().display()
    );

    meta_data.title = utils::convert_str(&meta_data.title, &config.converts)?;
    meta_data.author = utils::convert_str(&meta_data.author, &config.converts)?;
    if meta_data.description.is_some() {
        meta_data.description = Some(utils::convert_str(
            meta_data.description.unwrap(),
            &config.converts,
        )?);
    }
    meta_data.lang = utils::lang(&config.converts);

    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    let parser = Parser::new_ext(&markdown, options);

    let events = parser.collect::<Vec<_>>();
    let iter = events.into_par_iter().map(|event| match event {
        Event::Text(text) => {
            Event::Text(utils::convert_str(text, &config.converts).unwrap().into())
        }
        _ => event.to_owned(),
    });
    let events = iter.collect::<Vec<Event>>();

    let mut buf = String::with_capacity(markdown.len() + 1024);
    buf.push_str("---\n");
    buf.push_str(&serde_yaml::to_string(&meta_data)?);
    buf.push_str("...\n\n");
    pulldown_cmark_to_cmark::cmark(events.iter(), &mut buf)?;

    let novel_name = utils::convert_str(&meta_data.title, &config.converts)?;
    utils::remove_file_or_dir(&config.markdown_path)?;

    let path = utils::to_markdown_file_name(novel_name);
    fs::write(path, buf)?;

    info!("Time spent on `info`: {}", timing.elapsed()?);

    Ok(())
}
