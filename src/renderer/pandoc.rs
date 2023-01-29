use std::{env, fmt, path::PathBuf, sync::Arc};

use anyhow::Result;
use novel_api::Timing;
use once_cell::sync::OnceCell;
use tokio::{
    fs,
    task::{self, JoinHandle},
};
use tracing::{info, warn};

use crate::{
    cmd::Convert,
    utils::{self, Content, MetaData, Novel},
};

pub(crate) async fn generate_pandoc_markdown(novel: Novel, convert: &Vec<Convert>) -> Result<()> {
    let mut timing = Timing::new();

    let path = to_markdown_file_name(&novel.name);
    if path.is_file() {
        warn!("The epub output file already exists and will be overwritten");
    }

    let image_path = images_path()?;
    if image_path.is_dir() {
        warn!("The image output folder already exists and will be deleted");
        utils::remove_file_or_dir(image_path)?;
    }
    fs::create_dir_all(image_path).await?;

    let mut buf = String::with_capacity(4 * 1024 * 1024);

    write_metadata(&novel, &mut buf, convert).await?;
    write_introduction(&novel, &mut buf, convert).await?;
    write_chapters(&novel, &mut buf).await?;
    // last \n
    buf.pop();

    let handles = save_image(novel).await?;

    fs::write(path, &buf).await?;

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
        let base_path = env::current_dir()?;
        let path = base_path.join("images").join(format!(
            "cover.{}",
            utils::image_ext(novel.cover_image.read().await.as_ref().unwrap())
        ));
        let path = pathdiff::diff_paths(path, base_path).unwrap();

        cover_image = Some(path);
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
    let base_path = env::current_dir()?;
    let image_path = images_path()?;

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
                            let image_path = image_path.join(&image.file_name);
                            let image_path = pathdiff::diff_paths(image_path, &base_path).unwrap();

                            buf.write_str(&super::image_markdown_str(image_path))?;
                            buf.write_str("\n\n")?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn save_image(novel: Novel) -> Result<Vec<JoinHandle<Result<()>>>> {
    let image_path = images_path()?;

    let mut handles = Vec::new();

    for volume in novel.volumes {
        for chapter in volume.chapters {
            for index in 0..chapter.contents.read().await.len() {
                if let Content::Image(ref image) = chapter.contents.read().await[index] {
                    let path = image_path.join(&image.file_name);

                    let contents = Arc::clone(&chapter.contents);
                    handles.push(task::spawn_blocking(move || {
                        if let Content::Image(ref image) = contents.blocking_read()[index] {
                            image.content.save(path)?;
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
            let path = image_path.join(format!(
                "cover.{}",
                utils::image_ext(cover_image.blocking_read().as_ref().unwrap())
            ));
            cover_image.blocking_read().as_ref().unwrap().save(path)?;
            Ok(())
        }));
    }

    if handles.is_empty() {
        utils::remove_file_or_dir(image_path)?;
    }

    Ok(handles)
}

fn to_markdown_file_name<T>(novel_name: T) -> PathBuf
where
    T: AsRef<str>,
{
    let novel_name = novel_name.as_ref();

    if !sanitize_filename::is_sanitized(novel_name) {
        warn!("The output file name is invalid and has been modified");
    }

    let mut path = PathBuf::from(sanitize_filename::sanitize(novel_name));
    path.set_extension("md");

    path
}

fn images_path() -> Result<&'static PathBuf> {
    static IMAGE_PATH: OnceCell<PathBuf> = OnceCell::new();

    IMAGE_PATH.get_or_try_init(|| {
        let base_path = env::current_dir()?;
        Ok(base_path.join("images"))
    })
}
