use std::sync::Arc;

use image::DynamicImage;
use tokio::sync::RwLock;

#[must_use]
pub(crate) struct Novel {
    pub name: String,
    pub author_name: String,
    pub introduction: Option<Vec<String>>,
    pub cover_image: Arc<RwLock<Option<DynamicImage>>>,
    pub volumes: Vec<Volume>,
}

#[must_use]
pub(crate) struct Volume {
    pub title: String,
    pub chapters: Vec<Chapter>,
}

#[must_use]
pub(crate) struct Chapter {
    pub title: String,
    pub contents: Arc<RwLock<Vec<Content>>>,
}

#[must_use]
pub(crate) enum Content {
    Text(String),
    Image(Image),
}

#[must_use]
pub(crate) struct Image {
    pub file_name: String,
    pub content: DynamicImage,
}
