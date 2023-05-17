use std::{
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Result};
use novel_api::Timing;
use tokio::{
    fs,
    task::{self, JoinHandle},
};
use tracing::{info, warn};

use crate::{
    cmd::Convert,
    utils::{self, Content, MetaData, Novel, UNIX_LINE_BREAK, WINDOWS_LINE_BREAK},
};

pub async fn generate_pandoc_markdown(novel: Novel, convert: &Vec<Convert>) -> Result<()> {
    let mut timing = Timing::new();

    let output_dir_path = utils::to_novel_dir_name(&novel.name);
    if output_dir_path.is_dir() {
        warn!("The Pandoc output directory already exists and will be deleted");
        utils::remove_file_or_dir(&output_dir_path)?;
    }

    fs::create_dir(&output_dir_path).await?;
    let output_dir_path = dunce::canonicalize(output_dir_path)?;

    let markdown_file_path = output_dir_path.join(utils::to_markdown_file_name(&novel.name));

    let mut buf = String::with_capacity(4 * 1024 * 1024);

    write_metadata(&novel, &mut buf, convert).await?;
    write_introduction(&novel, &mut buf, convert).await?;
    write_chapters(&novel, &mut buf).await?;
    // last \n
    buf.pop();

    let handles = save_image(novel, output_dir_path).await?;

    if cfg!(windows) {
        buf = buf.replace(UNIX_LINE_BREAK, WINDOWS_LINE_BREAK);
    }
    fs::write(markdown_file_path, &buf).await?;

    for handle in handles {
        handle.await??;
    }

    info!(
        "Time spent on `generate pandoc markdown`: {}",
        timing.elapsed()?
    );

    Ok(())
}

async fn write_metadata<T>(novel: &Novel, mut buf: T, convert: &[Convert]) -> Result<()>
where
    T: fmt::Write,
{
    buf.write_str("---\n")?;

    let mut description = None;
    if let Some(ref introduction) = novel.introduction {
        description = Some(introduction.join("\n"));
    }

    let mut cover_image = None;
    if novel.cover_image.read().await.is_some() {
        let ext = utils::image_ext(novel.cover_image.read().await.as_ref().unwrap());

        if ext.is_ok() {
            cover_image = Some(PathBuf::from(format!("cover.{}", ext.unwrap())));
        } else {
            bail!("{}", ext.unwrap_err());
        }
    }

    let meta_data = MetaData {
        title: novel.name.clone(),
        author: novel.author_name.clone(),
        lang: utils::lang(convert),
        description,
        cover_image,
    };

    buf.write_str(&serde_yaml::to_string(&meta_data)?)?;
    buf.write_str("...\n\n")?;

    Ok(())
}

async fn write_introduction<T>(novel: &Novel, mut buf: T, convert: &Vec<Convert>) -> Result<()>
where
    T: fmt::Write,
{
    if let Some(ref introduction) = novel.introduction {
        buf.write_str(&format!("# {}\n\n", utils::convert_str("简介", convert)?))?;

        for line in introduction {
            buf.write_str(line)?;
            buf.write_str("\n\n")?;
        }
    }

    Ok(())
}

async fn write_chapters<T>(novel: &Novel, mut buf: T) -> Result<()>
where
    T: fmt::Write,
{
    for volume in &novel.volumes {
        if !volume.chapters.is_empty() {
            buf.write_str(&format!("# {}\n\n", volume.title))?;

            for chapter in &volume.chapters {
                buf.write_str(&format!("## {}\n\n", chapter.title))?;

                for content in chapter.contents.read().await.iter() {
                    match content {
                        Content::Text(line) => {
                            buf.write_str(line)?;
                            buf.write_str("\n\n")?;
                        }
                        Content::Image(image) => {
                            buf.write_str(&super::image_markdown_str(&image.file_name))?;
                            buf.write_str("\n\n")?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn save_image<T>(novel: Novel, path: T) -> Result<Vec<JoinHandle<Result<()>>>>
where
    T: AsRef<Path>,
{
    let path = path.as_ref().to_path_buf();
    let mut handles = Vec::new();

    for volume in novel.volumes {
        for chapter in volume.chapters {
            for index in 0..chapter.contents.read().await.len() {
                if let Content::Image(ref image) = chapter.contents.read().await[index] {
                    let image_path = path.join(&image.file_name);

                    let contents = Arc::clone(&chapter.contents);
                    handles.push(task::spawn_blocking(move || {
                        if let Content::Image(ref image) = contents.blocking_read()[index] {
                            image.content.save(image_path)?;
                        }

                        Ok(())
                    }));
                }
            }
        }
    }

    if novel.cover_image.read().await.is_some() {
        let cover_image = Arc::clone(&novel.cover_image);

        handles.push(task::spawn_blocking(move || {
            let ext = utils::image_ext(cover_image.blocking_read().as_ref().unwrap());

            if ext.is_ok() {
                let image_path = path.join(format!("cover.{}", ext.unwrap()));
                cover_image
                    .blocking_read()
                    .as_ref()
                    .unwrap()
                    .save(image_path)?;
            } else {
                bail!("{}", ext.unwrap_err());
            }

            Ok(())
        }));
    }

    Ok(handles)
}
