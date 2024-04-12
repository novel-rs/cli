mod mdbook;
mod pandoc;

use std::path::Path;
use std::{fs, thread};

use color_eyre::eyre::Ok;
use color_eyre::eyre::{bail, Result};
use image::DynamicImage;
use tracing::error;

use crate::utils;
use crate::utils::Content;
use crate::utils::Novel;

pub use self::mdbook::*;
pub use self::pandoc::*;

#[must_use]
fn image_markdown_str<T>(path: T) -> String
where
    T: AsRef<str>,
{
    format!("![]({})", path.as_ref())
}

fn cover_image_name(cover_image: &DynamicImage) -> Result<String> {
    let image_ext = utils::new_image_ext(cover_image);

    if image_ext.is_ok() {
        Ok(format!("cover.{}", image_ext.unwrap()))
    } else {
        bail!("{}", image_ext.unwrap_err());
    }
}

fn new_image_name(image: &DynamicImage, image_index: u16) -> Result<String> {
    Ok(format!(
        "{}.{}",
        utils::num_to_str(image_index),
        utils::new_image_ext(image)?
    ))
}

fn save_image<T>(novel: &Novel, image_dir_path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let image_dir_path = image_dir_path.as_ref();
    if !image_dir_path.exists() {
        fs::create_dir_all(image_dir_path)?;
    }

    if novel.cover_image.is_some() {
        let cover_image = novel.cover_image.as_ref().unwrap();
        let image_ext = utils::new_image_ext(cover_image);

        if image_ext.is_ok() {
            let image_ext = image_ext.unwrap();
            let image_path = image_dir_path.join(format!("cover.{}", image_ext));

            if image_ext == "webp" {
                novel_api::save_as_webp(cover_image, 75.0, image_path).unwrap_or_else(|err| {
                    error!("Failed to save cover image: {err}");
                });
            } else {
                cover_image.save(image_path).unwrap_or_else(|err| {
                    error!("Failed to save image: {err}");
                });
            }
        }
    }

    let mut images = Vec::new();
    for volume in &novel.volumes {
        for chapter in &volume.chapters {
            if chapter.contents.is_some() {
                for content in chapter.contents.as_ref().unwrap() {
                    if let Content::Image(image) = content {
                        images.push(image);
                    }
                }
            }
        }
    }

    thread::scope(|s| {
        let mut image_index = 1;

        for image in images {
            let image_ext = utils::new_image_ext(image);

            if image_ext.is_ok() {
                let image_ext = image_ext.unwrap();
                let image_name = format!("{}.{}", utils::num_to_str(image_index), image_ext);
                image_index += 1;

                let image_path = image_dir_path.join(image_name);

                if image_ext == "webp" {
                    s.spawn(|| {
                        novel_api::save_as_webp(image, 75.0, image_path).unwrap_or_else(|err| {
                            error!("Failed to save cover image: {err}");
                        });
                    });
                } else {
                    s.spawn(|| {
                        image.save(image_path).unwrap_or_else(|err| {
                            error!("Failed to save image: {err}");
                        });
                    });
                }
            }
        }
    });

    Ok(())
}
