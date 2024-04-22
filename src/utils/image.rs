use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use color_eyre::eyre::{bail, Result};
use crossterm::terminal;
use image::{codecs::jpeg::JpegEncoder, io::Reader, ColorType, DynamicImage};
use tracing::{error, info};

pub fn convert_image_ext<T>(image_path: T) -> Result<PathBuf>
where
    T: AsRef<Path>,
{
    let image_path = image_path.as_ref();

    if !image_path.is_file() {
        bail!("Image does not exist: {}", image_path.display());
    }

    let image_path = dunce::canonicalize(image_path)?;

    let image_ext = image_path.extension().unwrap().to_str().unwrap();
    if image_ext == "webp" {
        return Ok(image_path);
    }

    let image = Reader::open(&image_path)?.with_guessed_format()?.decode()?;

    match new_image_ext(&image) {
        Ok(new_image_ext) => {
            if image_ext != new_image_ext {
                let new_image_path = image_path.with_extension(new_image_ext);
                info!(
                    "Perform image format conversion: from `{}` to `{}`",
                    image_path.display(),
                    new_image_path.display()
                );

                if new_image_ext == "webp" {
                    novel_api::save_as_webp(&image, 75.0, &new_image_path)?;
                } else {
                    image.save(&new_image_path)?;
                }

                Ok(new_image_path)
            } else {
                Ok(image_path)
            }
        }
        Err(err) => {
            error!("Failed to convert image: {err}");
            Ok(image_path)
        }
    }
}

pub fn convert_image_file_stem<T, E>(image_path: T, new_image_stem: E) -> Result<PathBuf>
where
    T: AsRef<Path>,
    E: AsRef<str>,
{
    let image_path = image_path.as_ref();

    if !image_path.is_file() {
        bail!("Image does not exist: {}", image_path.display());
    }

    let image_path = dunce::canonicalize(image_path)?;
    let image_dir = image_path.parent().unwrap();
    let image_file_stem = image_path.file_stem().unwrap().to_str().unwrap();
    let image_ext = image_path.extension().unwrap().to_str().unwrap();

    if new_image_stem.as_ref() != image_file_stem {
        let new_image_path = image_dir.join(format!("{}.{image_ext}", new_image_stem.as_ref()));

        info!(
            "Perform image copy: from `{}` to `{}`",
            image_path.display(),
            new_image_path.display()
        );

        fs::copy(image_path, &new_image_path)?;

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

pub fn print_image(img: &DynamicImage) -> Result<()> {
    if is_iterm_supported() {
        let mut jpg = Vec::new();
        JpegEncoder::new_with_quality(&mut jpg, 75).encode_image(img)?;
        let data = base64_simd::STANDARD.encode_to_string(&jpg);

        let (width, height) = terminal_size();

        let mut stdout = io::stdout();
        writeln!(
            stdout,
            "\x1b]1337;File=inline=1;preserveAspectRatio=1;size={};width={};height={}:{data}\x07",
            jpg.len(),
            width / 2,
            height / 2
        )?;
        stdout.flush()?;
    }

    Ok(())
}

fn is_iterm_supported() -> bool {
    if let Ok(term) = env::var("TERM_PROGRAM") {
        if term.contains("iTerm") || term.contains("WezTerm") || term.contains("mintty") {
            return true;
        }
    }
    if let Ok(lc_term) = env::var("LC_TERMINAL") {
        if lc_term.contains("iTerm") || lc_term.contains("WezTerm") || lc_term.contains("mintty") {
            return true;
        }
    }
    false
}

pub fn terminal_size() -> (u16, u16) {
    terminal::size().unwrap_or((80, 24))
}
