use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Ok, Result};
use novel_api::Timing;
use serde::Serialize;
use tokio::{
    fs,
    sync::Semaphore,
    task::{self, JoinHandle},
};
use tracing::{info, warn};

use crate::{
    cmd::Convert,
    utils::{self, Content, Novel, Writer},
};

#[must_use]
#[derive(Debug, Serialize)]
struct Config {
    book: Book,
    output: Output,
}

#[must_use]
#[derive(Debug, Serialize)]
struct Book {
    title: String,
    description: Option<String>,
    authors: Vec<String>,
    language: String,
}

#[must_use]
#[derive(Debug, Serialize)]
struct Output {
    html: Html,
}

#[must_use]
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Html {
    no_section_label: bool,
}

pub(crate) async fn generate_mdbook(novel: Novel, convert: &Vec<Convert>) -> Result<()> {
    let mut timing = Timing::new();

    let base_path = to_mdbook_dir_name(&novel.name);
    if base_path.is_dir() {
        warn!("The mdBook output folder already exists and will be deleted");
        utils::remove_file_or_dir(&base_path)?;
    }

    fs::create_dir_all(&base_path).await?;

    generate_book_toml(&novel, &base_path, convert).await?;
    generate_summary(&novel, &base_path, convert).await?;
    generate_introduction(&novel, &base_path, convert).await?;
    generate_chapters(&novel, &base_path).await?;

    let handles = save_image(novel, &base_path).await?;
    for handle in handles {
        handle.await??;
    }

    info!("Time spent on `generate mdbook`: {}", timing.elapsed()?);

    Ok(())
}

async fn generate_book_toml<T>(novel: &Novel, base_path: T, convert: &[Convert]) -> Result<()>
where
    T: AsRef<Path>,
{
    let path = base_path.as_ref().join("book.toml");
    let mut writer = Writer::new(path).await?;

    let config = Config {
        book: Book {
            title: novel.name.clone(),
            description: novel.introduction.clone().map(|v| v.join("\n")),
            authors: vec![novel.author_name.clone()],
            language: super::lang(convert),
        },
        output: Output {
            html: Html {
                no_section_label: true,
            },
        },
    };

    writer.write(toml::to_string(&config)?).await?;
    writer.flush().await?;

    Ok(())
}

async fn generate_summary<T>(novel: &Novel, base_path: T, convert: &Vec<Convert>) -> Result<()>
where
    T: AsRef<Path>,
{
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

async fn generate_introduction<T>(novel: &Novel, base_path: T, convert: &Vec<Convert>) -> Result<()>
where
    T: AsRef<Path>,
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
    let semaphore = Arc::new(Semaphore::new(8));

    for volume in &novel.volumes {
        let volume_path = src_path.join(format!("volume{}", utils::num_to_str(volume_count)));
        volume_count += 1;

        if !volume.chapters.is_empty() {
            fs::create_dir_all(&volume_path).await?;

            let mut volume_writer = Writer::new(volume_path.join("README.md")).await?;
            volume_writer.writeln(format!("# {}", volume.title)).await?;
            volume_writer.flush().await?;

            for chapter in &volume.chapters {
                let mut chapter_path = volume_path
                    .clone()
                    .join(format!("chapter{}", utils::num_to_str(chapter_count)));
                chapter_path.set_extension("md");
                chapter_count += 1;

                let image_path = image_path.clone();
                let volume_path = volume_path.clone();
                let contents = Arc::clone(&chapter.contents);
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let title = chapter.title.clone();

                handles.push(tokio::spawn(async move {
                    let mut chapter_writer = Writer::new(chapter_path).await?;

                    chapter_writer.writeln(format!("# {title}")).await?;
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

async fn save_image<T>(novel: Novel, base_path: T) -> Result<Vec<JoinHandle<Result<()>>>>
where
    T: AsRef<Path>,
{
    let image_path = base_path.as_ref().join("src").join("images");
    fs::create_dir_all(&image_path).await?;

    let mut handles = Vec::new();
    let semaphore = Arc::new(Semaphore::new(32));

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
        let path = image_path.join("cover.webp");

        let cover_image = Arc::clone(&novel.cover_image);
        handles.push(task::spawn_blocking(move || {
            cover_image.blocking_read().as_ref().unwrap().save(path)?;
            Ok(())
        }));
    }

    if handles.is_empty() {
        utils::remove_file_or_dir(image_path)?;
    }

    Ok(handles)
}

fn to_mdbook_dir_name<T>(novel_name: T) -> PathBuf
where
    T: AsRef<str>,
{
    let novel_name = novel_name.as_ref();

    if !sanitize_filename::is_sanitized(novel_name) {
        warn!("The output file name is invalid and has been modified");
    }

    PathBuf::from(sanitize_filename::sanitize(novel_name))
}
