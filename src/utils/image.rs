use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{bail, Result};
use image::{ColorType, DynamicImage};
use tracing::info;

pub fn convert_image<T>(image_path: T, delete: bool) -> Result<PathBuf>
where
    T: AsRef<Path>,
{
    let image_path = image_path.as_ref();

    if !image_path.is_file() {
        bail!("Image does not exist: {}", image_path.display());
    }

    let image_path = dunce::canonicalize(image_path)?;
    let image_dir = image_path.parent().unwrap();

    let image_ext = image_path.extension().unwrap().to_str().unwrap();
    if image_ext == "webp" {
        return Ok(image_path);
    }

    let image = image::open(&image_path)?;
    let new_image_ext = new_image_ext(&image)?;

    if image_ext != new_image_ext {
        let new_image_path = image_path.with_extension(new_image_ext);
        info!(
            "Perform image format conversion: from `{}` to `{}`",
            image_path.display(),
            new_image_path.display()
        );

        if new_image_ext == "webp" {
            novel_api::save_as_webp(image, 75.0, &new_image_path)?;
        } else {
            image.save(&new_image_path)?;
        }

        if delete {
            fs::remove_file(image_path)?;
        } else {
            let file_stem = image_path.file_stem().unwrap().to_str().unwrap();
            let backup_file_name = format!("{}.old.{}", file_stem, image_ext);
            fs::rename(&image_path, image_dir.join(backup_file_name))?;
        }

        Ok(new_image_path)
    } else {
        Ok(image_path)
    }
}

pub fn new_image_ext(image: &DynamicImage) -> Result<&'static str> {
    match image.color() {
        ColorType::Rgb8 | ColorType::Rgba8 => Ok("webp"),
        ColorType::L8
        | ColorType::L16
        | ColorType::La8
        | ColorType::La16
        | ColorType::Rgb16
        | ColorType::Rgba16 => Ok("png"),
        other => bail!("This color type is not supported: {other:?}"),
    }
}
