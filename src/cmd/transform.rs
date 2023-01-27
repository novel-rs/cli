use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Args;
use fluent_templates::Loader;
use pulldown_cmark::{Event, Options, Parser};
use rayon::prelude::*;

use crate::{cmd::Convert, utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Debug, Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "transform_command").expect("`transform_command` does not exists"))]
pub struct Transform {
    #[arg(help = LOCALES.lookup(&LANG_ID, "markdown_path").expect("`markdown_path` does not exists"))]
    pub markdown_path: PathBuf,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").expect("`converts` does not exists"))]
    pub converts: Vec<Convert>,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "delete").expect("`delete` does not exists"))]
    pub delete: bool,
}

pub fn execute(config: Transform) -> Result<()> {
    let (mut meta_data, markdown) = utils::read_markdown(&config.markdown_path)?;

    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    let parser = Parser::new_ext(&markdown, options);

    let events = parser.collect::<Vec<Event>>();
    let iter = events.par_iter().map(|event| match event {
        Event::Text(text) => {
            Event::Text(utils::convert_str(text, &config.converts).unwrap().into())
        }
        _ => event.to_owned(),
    });
    let events = iter.collect::<Vec<Event>>();

    meta_data.title = utils::convert_str(&meta_data.title, &config.converts)?;
    meta_data.author = utils::convert_str(&meta_data.author, &config.converts)?;
    if meta_data.description.is_some() {
        meta_data.description = Some(utils::convert_str(
            meta_data.description.unwrap(),
            &config.converts,
        )?);
    }
    meta_data.lang = utils::lang(&config.converts);

    let mut buf = String::with_capacity(markdown.len() + 1024);
    buf.push_str("---\n");
    buf.push_str(&serde_yaml::to_string(&meta_data)?);
    buf.push_str("...\n\n");
    pulldown_cmark_to_cmark::cmark(events.iter(), &mut buf)?;

    let novel_name = utils::file_stem(&config.markdown_path)?
        .display()
        .to_string();
    let new_novel_name = utils::convert_str(&novel_name, &config.converts)?;

    if config.delete {
        utils::remove_file_or_dir(&config.markdown_path)?;
    } else if novel_name == new_novel_name {
        let mut backup_file = config.markdown_path.clone();
        backup_file.set_extension("backup");
        fs::rename(&config.markdown_path, backup_file)?;
    }

    fs::write(format!("{new_novel_name}.md"), buf)?;

    Ok(())
}
