use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::Result;
use fluent_templates::Loader;

use crate::{
    renderer,
    utils::{self, Chapter, Content, Novel, Volume},
    LANG_ID, LOCALES,
};

use super::Convert;

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "template_command"))]
pub struct Template {
    #[arg(help = LOCALES.lookup(&LANG_ID, "novel_name"))]
    pub novel_name: String,

    #[arg(long, help = LOCALES.lookup(&LANG_ID, "cover_image"))]
    pub cover_image: Option<PathBuf>,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts"))]
    pub converts: Vec<Convert>,
}

pub fn execute(config: Template) -> Result<()> {
    let chapter = Chapter {
        id: 0,
        title: String::from("Chapter 1"),
        contents: vec![
            Content::Text(String::from("◇◇◇")),
            Content::Text(String::from("![](001.webp)")),
        ],
    };

    let volume = Volume {
        title: String::from("Volume 1"),
        chapters: vec![chapter],
    };

    let cover_image = if config.cover_image.is_some() {
        let image = image::open(config.cover_image.unwrap())?;
        Some(image)
    } else {
        None
    };

    let mut novel = Novel {
        name: config.novel_name,
        author_name: String::from("TODO"),
        introduction: Some(vec![String::from("line 1"), String::from("line 2")]),
        cover_image,
        volumes: vec![volume],
    };

    utils::convert(&mut novel, &config.converts)?;

    renderer::generate_pandoc_markdown(novel, config.converts)?;

    Ok(())
}
