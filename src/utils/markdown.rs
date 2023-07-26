use std::{
    fs,
    panic::{self, RefUnwindSafe},
    path::{Path, PathBuf},
    sync::Mutex,
};

use color_eyre::eyre::{bail, ensure, Result};
use novel_api::Timing;
use pulldown_cmark::{Event, Options, Parser, Tag};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::debug;

use crate::{
    cmd::Convert,
    utils::{self, LINE_BREAK},
};

#[must_use]
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MetaData {
    pub title: String,
    pub author: String,
    pub lang: String,
    pub description: Option<String>,
    pub cover_image: Option<PathBuf>,
}

impl MetaData {
    pub fn lang_is_ok(&self) -> bool {
        self.lang == "zh-Hant" || self.lang == "zh-Hans"
    }

    pub fn cover_image_is_ok(&self) -> bool {
        !self
            .cover_image
            .as_ref()
            .is_some_and(|path| !path.is_file())
    }
}

pub fn read_markdown<T>(markdown_path: T) -> Result<(MetaData, String)>
where
    T: AsRef<Path>,
{
    let mut timing = Timing::new();

    let markdown_path = markdown_path.as_ref();

    let bytes = fs::read(markdown_path)?;
    let markdown = simdutf8::basic::from_utf8(&bytes)?;
    utils::verify_line_break(markdown)?;

    ensure!(
        markdown.starts_with("---"),
        "The markdown format is incorrect, it should start with `---`"
    );

    if let Some(index) = markdown.find(format!("{0}...{0}", LINE_BREAK).as_str()) {
        let yaml = &markdown[3 + LINE_BREAK.len()..index];

        let meta_data: MetaData = serde_yaml::from_str(yaml)?;
        let markdown = markdown[index + 3 + LINE_BREAK.len() * 2..].to_string();

        debug!("Time spent on `read_markdown`: {}", timing.elapsed()?);

        Ok((meta_data, markdown))
    } else {
        bail!("The markdown format is incorrect, it should end with `...`");
    }
}

pub fn to_markdown_events<T, E>(
    markdown: &str,
    converts: T,
    input_dir: E,
    delete: bool,
) -> Result<Vec<Event>>
where
    T: AsRef<[Convert]> + Sync + RefUnwindSafe,
    E: AsRef<Path> + Sync + RefUnwindSafe,
{
    let mut timing = Timing::new();

    let parser = Parser::new_ext(markdown, Options::empty());
    let events = parser.collect::<Vec<_>>();

    let result = panic::catch_unwind(|| {
        let iter = events.into_par_iter().map(|event| match event {
            Event::Text(text) => {
                Event::Text(utils::convert_str(text, converts.as_ref()).unwrap().into())
            }
            Event::Start(Tag::Image(link_type, path, title)) => {
                let new_image_path =
                    super::convert_image(input_dir.as_ref().join(path.into_string()), delete)
                        .unwrap();

                Event::Start(Tag::Image(
                    link_type,
                    new_image_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                        .into(),
                    title,
                ))
            }
            _ => event.to_owned(),
        });

        iter.collect::<Vec<Event>>()
    });

    if let Err(error) = result {
        bail!("{error:?}")
    } else {
        debug!("Time spent on `to_markdown_events`: {}", timing.elapsed()?);

        Ok(result.unwrap())
    }
}

pub fn read_markdown_to_images<T>(markdown_path: T) -> Result<Vec<PathBuf>>
where
    T: AsRef<Path>,
{
    let (metadata, markdown) = read_markdown(markdown_path)?;

    let parser = Parser::new_ext(&markdown, Options::empty());
    let events = parser.collect::<Vec<_>>();

    let result = Mutex::new(Vec::new());

    events.into_par_iter().for_each(|event| {
        if let Event::Start(Tag::Image(_, path, _)) = event {
            result
                .lock()
                .unwrap()
                .push(PathBuf::from(&path.to_string()));
        }
    });

    let mut result = result.lock().unwrap().to_vec();
    if metadata.cover_image.is_some() {
        result.push(metadata.cover_image.unwrap())
    }

    Ok(result)
}
