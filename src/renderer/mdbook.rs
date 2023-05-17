use std::{path::Path, sync::Arc};

use anyhow::{bail, Ok, Result};
use novel_api::Timing;
use serde::Serialize;
use tokio::{fs, sync::Semaphore, task};
use tracing::{info, warn};

use crate::{
    cmd::Convert,
    utils::{self, Content, Novel, Writer, UNIX_LINE_BREAK, WINDOWS_LINE_BREAK},
};

#[must_use]
#[derive(Serialize)]
struct Config {
    book: Book,
    output: Output,
}

#[must_use]
#[derive(Serialize)]
struct Book {
    title: String,
    description: Option<String>,
    authors: Vec<String>,
    language: String,
}

#[must_use]
#[derive(Serialize)]
struct Output {
    html: Html,
}

#[must_use]
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct Html {
    no_section_label: bool,
}

pub async fn generate_mdbook<T>(novel: Novel, convert: T) -> Result<()>
where
    T: AsRef<[Convert]>,
{
    let mut timing = Timing::new();

    let convert = convert.as_ref();

    let output_dir_path = utils::to_novel_dir_name(&novel.name);
    if output_dir_path.is_dir() {
        warn!("The mdBook output folder already exists and will be deleted");
        utils::remove_file_or_dir(&output_dir_path)?;
    }

    fs::create_dir_all(&output_dir_path).await?;
    let base_path = dunce::canonicalize(output_dir_path)?;

    generate_book_toml(&novel, &base_path, convert).await?;
    generate_summary(&novel, &base_path, convert).await?;
    generate_introduction(&novel, &base_path, convert).await?;
    generate_chapters(&novel, &base_path).await?;
    generate_image(novel, &base_path).await?;

    info!("Time spent on `generate mdBook`: {}", timing.elapsed()?);

    Ok(())
}

async fn generate_book_toml<T, E>(novel: &Novel, base_path: T, convert: E) -> Result<()>
where
    T: AsRef<Path>,
    E: AsRef<[Convert]>,
{
    let config = Config {
        book: Book {
            title: novel.name.clone(),
            description: novel.introduction.clone().map(|v| v.join(UNIX_LINE_BREAK)),
            authors: vec![novel.author_name.clone()],
            language: utils::lang(convert),
        },
        output: Output {
            html: Html {
                no_section_label: true,
            },
        },
    };

    let mut buf = toml::to_string(&config)?;
    if cfg!(windows) {
        buf = buf.replace(UNIX_LINE_BREAK, WINDOWS_LINE_BREAK);
    }

    let path = base_path.as_ref().join("book.toml");
    let mut writer = Writer::new(path).await?;

    writer.write(buf).await?;
    writer.flush().await?;

    Ok(())
}

async fn generate_summary<T, E>(novel: &Novel, base_path: T, convert: E) -> Result<()>
where
    T: AsRef<Path>,
    E: AsRef<[Convert]>,
{
    let convert = convert.as_ref();

    let mut path = base_path.as_ref().join("src");
    fs::create_dir_all(&path).await?;
    path.push("SUMMARY.md");

    let mut writer = Writer::new(path).await?;

    writer
        .writeln(format!("# {}", utils::convert_str("目录", convert)?))
        .await?;
    writer.ln().await?;

    if novel.introduction.is_some() {
        writer
            .writeln(format!(
                "- [{}](introduction.md)",
                utils::convert_str("简介", convert)?
            ))
            .await?;
        writer.ln().await?;
    }

    let mut volume_count = 1;
    let mut chapter_count = 1;

    for volume in &novel.volumes {
        let volume_dir = format!("volume{}", utils::num_to_str(volume_count));
        volume_count += 1;

        writer
            .writeln(format!("- [{}]({}/README.md)", volume.title, volume_dir))
            .await?;

        for chapter in &volume.chapters {
            let chapter_file_name = format!("chapter{}.md", utils::num_to_str(chapter_count));
            chapter_count += 1;

            writer
                .writeln(format!(
                    "  - [{}]({}/{})",
                    chapter.title, volume_dir, chapter_file_name
                ))
                .await?;
        }
    }

    writer.flush().await?;

    Ok(())
}

async fn generate_introduction<T, E>(novel: &Novel, base_path: T, convert: E) -> Result<()>
where
    T: AsRef<Path>,
    E: AsRef<[Convert]>,
{
    if let Some(ref introduction) = novel.introduction {
        let mut path = base_path.as_ref().join("src");
        fs::create_dir_all(&path).await?;
        path.push("introduction.md");

        let mut writer = Writer::new(path).await?;

        writer
            .writeln(format!("# {}", utils::convert_str("简介", convert)?))
            .await?;
        writer.ln().await?;

        for line in introduction {
            writer.writeln(line).await?;
            writer.ln().await?;
        }

        writer.flush().await?;
    }

    Ok(())
}

async fn generate_chapters<T>(novel: &Novel, base_path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let src_path = base_path.as_ref().join("src");
    fs::create_dir_all(&src_path).await?;

    let image_path = src_path.join("images");

    let mut volume_count = 1;
    let mut chapter_count = 1;

    let mut handles = Vec::new();
    let semaphore = Arc::new(Semaphore::new(num_cpus::get()));

    for volume in &novel.volumes {
        let volume_path = src_path.join(format!("volume{}", utils::num_to_str(volume_count)));
        volume_count += 1;

        if !volume.chapters.is_empty() {
            fs::create_dir_all(&volume_path).await?;

            let mut volume_writer = Writer::new(volume_path.join("README.md")).await?;
            volume_writer.writeln(format!("# {}", volume.title)).await?;
            volume_writer.flush().await?;

            for chapter in &volume.chapters {
                let chapter_path = volume_path
                    .join(format!("chapter{}", utils::num_to_str(chapter_count)))
                    .with_extension("md");
                chapter_count += 1;

                let image_path = image_path.clone();
                let volume_path = volume_path.clone();
                let contents = Arc::clone(&chapter.contents);
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let title = format!("# {}", chapter.title);

                handles.push(tokio::spawn(async move {
                    let mut chapter_writer = Writer::new(chapter_path).await?;

                    chapter_writer.writeln(title).await?;
                    chapter_writer.ln().await?;

                    for content in contents.read().await.iter() {
                        match content {
                            Content::Text(line) => {
                                chapter_writer.writeln(line).await?;
                                chapter_writer.ln().await?;
                            }
                            Content::Image(image) => {
                                let image_path = image_path.join(&image.file_name);
                                let image_path =
                                    pathdiff::diff_paths(image_path, &volume_path).unwrap();

                                chapter_writer
                                    .writeln(super::image_markdown_str(image_path))
                                    .await?;
                                chapter_writer.ln().await?;
                            }
                        }
                    }

                    chapter_writer.flush().await?;

                    drop(permit);

                    Ok(())
                }));
            }
        }
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

async fn generate_image<T>(novel: Novel, base_path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let image_path = base_path.as_ref().join("src").join("images");
    fs::create_dir_all(&image_path).await?;

    let mut handles = Vec::new();
    let semaphore = Arc::new(Semaphore::new(num_cpus::get()));

    for volume in novel.volumes {
        for chapter in volume.chapters {
            for index in 0..chapter.contents.read().await.len() {
                if let Content::Image(ref image) = chapter.contents.read().await[index] {
                    let path = image_path.join(&image.file_name);
                    let permit = semaphore.clone().acquire_owned().await.unwrap();
                    let contents = Arc::clone(&chapter.contents);

                    handles.push(task::spawn_blocking(move || {
                        if let Content::Image(ref image) = contents.blocking_read()[index] {
                            image.content.save(path)?;
                            drop(permit);
                        }

                        Ok(())
                    }));
                }
            }
        }
    }

    if novel.cover_image.read().await.is_some() {
        let cover_image = Arc::clone(&novel.cover_image);
        let image_path = image_path.clone();

        handles.push(task::spawn_blocking(move || {
            let ext = utils::image_ext(cover_image.blocking_read().as_ref().unwrap());

            if ext.is_ok() {
                let path = image_path.join(format!("cover.{}", ext.unwrap()));
                cover_image.blocking_read().as_ref().unwrap().save(path)?;
            } else {
                bail!("{}", ext.unwrap_err());
            }

            Ok(())
        }));
    }

    if handles.is_empty() {
        utils::remove_file_or_dir(image_path)?;
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}
