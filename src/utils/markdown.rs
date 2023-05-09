use std::{
    fs,
    panic::{self, RefUnwindSafe},
    path::{Path, PathBuf},
};

use anyhow::{bail, ensure, Result};
use pulldown_cmark::{Event, Options, Parser};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{cmd::Convert, utils};

#[must_use]
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
        !novel_api::is_some_and(self.cover_image.as_ref(), |path| !path.is_file())
    }
}

pub fn ensure_markdown_file<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    ensure!(
        path.try_exists()?,
        "File `{}` does not exist",
        path.display()
    );
    ensure!(path.is_file(), "`{}` is not file", path.display());
    ensure!(
        novel_api::is_some_and(path.extension(), |extension| extension == "md"),
        "File `{}` is not markdown file",
        path.display()
    );

    Ok(())
}

pub fn read_markdown<T>(markdown_path: T) -> Result<(MetaData, String)>
where
    T: AsRef<Path>,
{
    let markdown_path = markdown_path.as_ref();

    let bytes = fs::read(markdown_path)?;
    let markdown = simdutf8::basic::from_utf8(&bytes)?;
    utils::verify_line_break(markdown)?;

    ensure!(
        markdown.starts_with("---"),
        "The markdown format is incorrect, it should start with `---`"
    );

    if let Some(index) = markdown.find(format!("{0}...{0}", utils::LINE_BREAK).as_str()) {
        let yaml = &markdown[3 + utils::LINE_BREAK.len()..index];

        let meta_data: MetaData = serde_yaml::from_str(yaml)?;
        let markdown = markdown[index + 3 + utils::LINE_BREAK.len() * 2..].to_string();

        Ok((meta_data, markdown))
    } else {
        bail!("The markdown format is incorrect, it should end with `...`");
    }
}

pub fn to_events<T>(markdown: &str, converts: T) -> Result<Vec<Event>>
where
    T: AsRef<[Convert]> + Sync + RefUnwindSafe,
{
    let parser = Parser::new_ext(markdown, Options::empty());
    let events = parser.collect::<Vec<_>>();

    let result = panic::catch_unwind(|| {
        let iter = events.into_par_iter().map(|event| match event {
            Event::Text(text) => {
                Event::Text(utils::convert_str(text, converts.as_ref()).unwrap().into())
            }
            _ => event.to_owned(),
        });

        iter.collect::<Vec<Event>>()
    });

    if let Err(error) = result {
        bail!("`convert_str` execution failed: {error:?}")
    } else {
        Ok(result.unwrap())
    }
}
