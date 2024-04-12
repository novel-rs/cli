use image::DynamicImage;

#[must_use]
pub struct Novel {
    pub name: String,
    pub author_name: String,
    pub introduction: Option<Vec<String>>,
    pub cover_image: Option<DynamicImage>,
    pub volumes: Vec<Volume>,
}

#[must_use]
pub struct Volume {
    pub title: String,
    pub chapters: Vec<Chapter>,
}

#[must_use]
pub struct Chapter {
    pub id: u32,
    pub title: String,
    pub contents: Option<Vec<Content>>,
}

#[must_use]
pub enum Content {
    Text(String),
    Image(DynamicImage),
}
