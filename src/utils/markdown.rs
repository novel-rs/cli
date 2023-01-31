use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

use crate::utils;

#[must_use]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct MetaData {
    pub title: String,
    pub author: String,
    pub lang: String,
    pub description: Option<String>,
    pub cover_image: Option<PathBuf>,
}

impl MetaData {
    pub(crate) fn lang_is_ok(&self) -> bool {
        self.lang == "zh-Hant" || self.lang == "zh-Hans"
    }

    pub(crate) fn cover_image_is_ok(&self) -> bool {
        !novel_api::is_some_and(self.cover_image.as_ref(), |cover_image| {
            !cover_image.is_file()
        })
    }
}

pub(crate) fn read_markdown<T>(markdown_path: T) -> Result<(MetaData, String)>
where
    T: AsRef<Path>,
{
    let markdown_path = markdown_path.as_ref();
    ensure!(
        utils::is_markdown(markdown_path),
        "The input file is not in markdown format"
    );

    let bytes = fs::read(markdown_path)?;
    let markdown = simdutf8::basic::from_utf8(&bytes)?;

    ensure!(
        markdown.starts_with("---"),
        "Markdown format is incorrectnot: start with `---`"
    );

    let index = markdown.find("\n...\n").unwrap();
    let yaml = &markdown[4..index];

    let meta_data: MetaData = serde_yaml::from_str(yaml).unwrap();
    let markdown = markdown[index + 5..].to_string();

    Ok((meta_data, markdown))
}
