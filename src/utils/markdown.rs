use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{bail, Result};
use pulldown_cmark::{Event, MetadataBlockKind, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[must_use]
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Metadata {
    pub title: String,
    pub author: String,
    pub lang: String,
    pub description: Option<String>,
    pub cover_image: Option<PathBuf>,
}

impl Metadata {
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

pub fn get_metadata_from_file<T>(markdown_path: T) -> Result<Metadata>
where
    T: AsRef<Path>,
{
    let bytes = fs::read(markdown_path)?;
    let markdown = simdutf8::basic::from_utf8(&bytes)?;

    let mut parser = Parser::new_ext(markdown, Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    get_metadata(&mut parser)
}

pub fn get_metadata(parser: &mut Parser) -> Result<Metadata> {
    let event = parser.next();
    if event.is_none()
        || !matches!(
            event.unwrap(),
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle))
        )
    {
        bail!("Markdown files should start with a metadata block")
    }

    let metadata: Metadata;
    if let Some(Event::Text(text)) = parser.next() {
        metadata = serde_yaml::from_str(&text)?;
    } else {
        bail!("Metadata block content does not exist")
    }

    let event = parser.next();
    if event.is_none()
        || !matches!(
            event.unwrap(),
            Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle))
        )
    {
        bail!("Metadata block should end with `---` or `...`")
    }

    Ok(metadata)
}

pub fn read_markdown_to_images<T>(markdown_path: T) -> Result<Vec<PathBuf>>
where
    T: AsRef<Path>,
{
    let bytes = fs::read(markdown_path)?;
    let markdown = simdutf8::basic::from_utf8(&bytes)?;

    let mut parser = Parser::new_ext(markdown, Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    let metadata = get_metadata(&mut parser)?;

    let parser = parser.filter_map(|event| {
        if let Event::Start(Tag::Image { dest_url, .. }) = event {
            Some(PathBuf::from(dest_url.as_ref()))
        } else {
            None
        }
    });

    let mut result: Vec<PathBuf> = parser.collect();
    if metadata.cover_image.is_some() {
        result.push(metadata.cover_image.unwrap())
    }

    Ok(result)
}
