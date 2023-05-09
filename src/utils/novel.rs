use std::sync::Arc;

use image::DynamicImage;
use tokio::sync::RwLock;

#[must_use]
pub struct Novel {
    pub name: String,
    pub author_name: String,
    pub introduction: Option<Vec<String>>,
    pub cover_image: Arc<RwLock<Option<DynamicImage>>>,
    pub volumes: Vec<Volume>,
}

#[must_use]
pub struct Volume {
    pub title: String,
    pub chapters: Vec<Chapter>,
}

#[must_use]
pub struct Chapter {
    pub title: String,
    pub contents: Arc<RwLock<Vec<Content>>>,
}

#[must_use]
pub enum Content {
    Text(String),
    Image(Image),
}

#[must_use]
pub struct Image {
    pub file_name: String,
    pub content: DynamicImage,
}
