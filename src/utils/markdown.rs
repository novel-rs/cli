use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct MetaData {
    pub title: String,
    pub author: String,
    pub lang: String,
    pub description: Option<String>,
    pub cover_image: Option<PathBuf>,
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

    ensure!(
        meta_data.lang == "zh-Hant" || meta_data.lang == "zh-Hans",
        "The lang field must be zh-Hant or zh-Hans"
    );
    if meta_data.cover_image.is_some() {
        ensure!(
            meta_data.cover_image.as_ref().unwrap().exists(),
            "Cover image does not exist"
        );
    }

    let markdown = markdown[index + 5..].to_string();

    Ok((meta_data, markdown))
}
