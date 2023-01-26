use std::sync::Arc;

use image::DynamicImage;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct Novel {
    pub name: String,
    pub author_name: String,
    pub introduction: Option<Vec<String>>,
    pub cover_image: Arc<RwLock<Option<DynamicImage>>>,
    pub volumes: Vec<Volume>,
}

#[derive(Debug)]
pub struct Volume {
    pub title: String,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug)]
pub struct Chapter {
    pub title: String,
    pub contents: Arc<RwLock<Vec<Content>>>,
}

#[derive(Debug)]
pub enum Content {
    Text(String),
    Image(Image),
}

#[derive(Debug)]
pub struct Image {
    pub name: String,
    pub content: DynamicImage,
}
