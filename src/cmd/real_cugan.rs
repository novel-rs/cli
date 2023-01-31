use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};

use anyhow::{bail, Ok, Result};
use bytes::BytesMut;
use clap::{value_parser, Args};
use fluent_templates::Loader;
use image::io::Reader;
use novel_api::Timing;
use parking_lot::RwLock;
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, BufReader},
    process::{Child, Command},
    sync::Semaphore,
};
use tracing::info;
use walkdir::WalkDir;

use crate::{
    utils::{self, ProgressBar},
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(about = LOCALES.lookup(&LANG_ID, "real_cugan_command").unwrap())]
pub struct RealCugan {
    #[arg(help = LOCALES.lookup(&LANG_ID, "image_path").unwrap())]
    pub image_path: Option<PathBuf>,

    #[arg(short, long, default_value_t = 4, value_parser = value_parser!(u8).range(1..=16),
        help = LOCALES.lookup(&LANG_ID, "maximum_concurrency").unwrap())]
    pub maximum_concurrency: u8,
}

pub async fn execute(config: RealCugan) -> Result<()> {
    let mut timing = Timing::new();

    let mut handles = Vec::new();
    let mut to_remove = Vec::new();

    let semaphore = Arc::new(Semaphore::new(config.maximum_concurrency as usize));
    let image_paths = image_paths(config).await?;
    let pb = Arc::new(RwLock::new(ProgressBar::new(image_paths.len())));

    let curr_path = env::current_dir()?;

    for input_path in image_paths {
        let image = Reader::open(&input_path)?.decode()?;
        let scale = calc_scale(image.width(), image.height());

        let mut output_path = input_path.clone();
        output_path.set_extension(utils::image_ext(&image));

        if input_path != output_path {
            to_remove.push(input_path.clone());
        }

        info!(
            "Run realcugan-ncnn-vulkan with `{}`, {}x{}, scale: {}",
            input_path.display(),
            image.width(),
            image.height(),
            scale
        );

        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pb = Arc::clone(&pb);

        let absolute_path = fs::canonicalize(&input_path).await?;
        let diff_path = pathdiff::diff_paths(absolute_path, &curr_path).unwrap();

        handles.push(tokio::spawn(async move {
            pb.write().inc(diff_path.display().to_string());
            create_child(input_path, output_path, scale)?.wait().await?;

            drop(permit);

            Ok(())
        }));
    }

    for handle in handles {
        handle.await??;
    }

    pb.write().finish();

    for path in to_remove {
        utils::remove_file_or_dir(path)?;
    }

    info!("Time spent on `info`: {}", timing.elapsed()?);

    Ok(())
}

async fn image_paths(config: RealCugan) -> Result<Vec<PathBuf>> {
    let curr_path = env::current_dir()?;

    let image_path = if config.image_path.is_some() {
        config.image_path.unwrap()
    } else {
        curr_path
    };

    let mut result = Vec::new();

    for entry in WalkDir::new(image_path).max_depth(1) {
        let input_path = entry?.path().to_path_buf();

        if is_image(&input_path).await? {
            result.push(input_path);
        }
    }

    if result.is_empty() {
        bail!("There is no image in this directory");
    }

    Ok(result)
}

async fn is_image<T>(path: T) -> Result<bool>
where
    T: AsRef<Path>,
{
    if !path.as_ref().is_file() {
        return Ok(false);
    }

    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut buffer = BytesMut::with_capacity(1024);

    reader.read_buf(&mut buffer).await?;

    Ok(infer::is_image(&buffer))
}

// 5k: 5120*2880=  14745600
// 4k: 3840*2160=   8294400
// 2k: 2560*1440=   3686400
// 1080p: 1920×1080=2073600
#[must_use]
#[inline]
const fn calc_scale(width: u32, height: u32) -> u8 {
    let pixel = width * height;
    let n = (5120 * 2880 / pixel) as u8;

    if n >= 16 {
        4
    } else if n >= 9 {
        3
    } else {
        2
    }
}

fn create_child<T, E>(input_path: T, output_path: E, scale: u8) -> Result<Child>
where
    T: AsRef<Path>,
    E: AsRef<Path>,
{
    let child = Command::new("realcugan-ncnn-vulkan")
        .arg("-i")
        .arg(input_path.as_ref())
        .arg("-o")
        .arg(output_path.as_ref())
        .arg("-s")
        .arg(scale.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(child)
}
