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

    let image = image::open(&image_path)?;
    let new_image_ext = image_ext(&image)?;
    let old_image_ext = image_path.extension().unwrap().to_str().unwrap();

    if old_image_ext != new_image_ext {
        let new_image_path = image_path.with_extension(new_image_ext);
        image.save(&new_image_path)?;

        info!(
            "Perform image format conversion: from `{}` to `{}`",
            image_path.display(),
            new_image_path.display()
        );

        if delete {
            fs::remove_file(image_path)?;
        } else {
            let old_file_stem = image_path.file_stem().unwrap().to_str().unwrap();
            let backup_file_name = format!("{}.old.{}", old_file_stem, old_image_ext);
            fs::rename(&image_path, image_dir.join(backup_file_name))?;
        }

        Ok(new_image_path)
    } else {
        Ok(image_path)
    }
}

#[inline]
pub fn image_ext(image: &DynamicImage) -> Result<String> {
    match image.color() {
        ColorType::Rgb8 | ColorType::Rgba8 => Ok(String::from("webp")),
        ColorType::L8
        | ColorType::L16
        | ColorType::La8
        | ColorType::La16
        | ColorType::Rgb16
        | ColorType::Rgba16 => Ok(String::from("png")),
        other => bail!("This color type is not supported: {:?}", other),
    }
}
