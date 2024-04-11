use std::path::Path;

use color_eyre::eyre::Result;
use novel_api::Timing;
use serde::Serialize;
use tokio::fs;
use tracing::{debug, error, warn};

use crate::{
    cmd::Convert,
    utils::{self, Content, Lang, Novel, Writer},
};

#[must_use]
#[derive(Serialize)]
struct Config<'a> {
    book: Book<'a>,
    output: Output,
}

#[must_use]
#[derive(Serialize)]
struct Book<'a> {
    title: &'a str,
    description: Option<String>,
    authors: Vec<&'a str>,
    language: Lang,
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

    assert!(!novel.name.is_empty(), "The novel name is empty");

    let output_dir_path = utils::to_novel_dir_name(&novel.name);
    if output_dir_path.is_dir() {
        warn!("The mdBook output directory already exists and will be deleted");
        utils::remove_file_or_dir(&output_dir_path)?;
    }

    fs::create_dir_all(&output_dir_path).await?;
    let base_path = dunce::canonicalize(output_dir_path)?;

    generate_book_toml(&novel, &base_path, &convert).await?;
    generate_summary(&novel, &base_path, &convert).await?;
    generate_cover(&novel, &base_path, &convert).await?;
    generate_introduction(&novel, &base_path, &convert).await?;
    generate_chapters(&novel, &base_path).await?;

    let image_path = base_path.join("src").join("images");
    super::save_image(&novel, image_path)?;

    debug!("Time spent on `generate mdBook`: {}", timing.elapsed()?);

    Ok(())
}

async fn generate_book_toml<T, E>(novel: &Novel, base_path: T, convert: E) -> Result<()>
where
    T: AsRef<Path>,
    E: AsRef<[Convert]>,
{
    let config = Config {
        book: Book {
            title: novel.name.as_str(),
            description: novel.introduction.as_ref().map(|v| v.join("\n")),
            authors: vec![novel.author_name.as_str()],
            language: utils::lang(convert),
        },
        output: Output {
            html: Html {
                no_section_label: true,
            },
        },
    };

    let buf = toml::to_string(&config)?;

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
    let path = base_path.as_ref().join("src").join("SUMMARY.md");
    let mut writer = Writer::new(path).await?;

    writer
        .writeln(format!(
            "# {}",
            utils::convert_str("目录", &convert, false)?
        ))
        .await?;
    writer.ln().await?;

    if novel.cover_image.is_some() {
        writer
            .writeln(format!(
                "- [{}](cover.md)",
                utils::convert_str("封面", &convert, false)?
            ))
            .await?;
        writer.ln().await?;
    }

    if novel.introduction.is_some() {
        writer
            .writeln(format!(
                "- [{}](introduction.md)",
                utils::convert_str("简介", &convert, false)?
            ))
            .await?;
        writer.ln().await?;
    }

    let mut volume_count = 1;
    let mut chapter_count = 1;

    for volume in &novel.volumes {
        let volume_dir = format!("volume{}", utils::num_to_str(volume_count));
        volume_count += 1;

        if !volume.chapters.is_empty() {
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
    }

    writer.flush().await?;

    Ok(())
}

async fn generate_cover<T, E>(novel: &Novel, base_path: T, convert: E) -> Result<()>
where
    T: AsRef<Path>,
    E: AsRef<[Convert]>,
{
    if let Some(ref cover_image) = novel.cover_image {
        let path = base_path.as_ref().join("src").join("cover.md");
        let mut writer = Writer::new(&path).await?;

        writer
            .writeln(format!("# {}", utils::convert_str("封面", convert, false)?))
            .await?;

        match super::cover_image_name(cover_image) {
            Ok(cover_image_name) => {
                let image_path = path.join("images").join(cover_image_name);
                let image_path = pathdiff::diff_paths(image_path, &path).unwrap();
                let image_path_str = image_path.display().to_string().replace('\\', "/");

                writer.ln().await?;
                writer
                    .writeln(&super::image_markdown_str(image_path_str))
                    .await?;
            }
            Err(err) => error!("Failed to get cover image name: {err}"),
        }

        writer.flush().await?;
    }

    Ok(())
}

async fn generate_introduction<T, E>(novel: &Novel, base_path: T, convert: E) -> Result<()>
where
    T: AsRef<Path>,
    E: AsRef<[Convert]>,
{
    if let Some(ref introduction) = novel.introduction {
        let path = base_path.as_ref().join("src").join("introduction.md");
        let mut writer = Writer::new(path).await?;

        writer
            .writeln(format!("# {}", utils::convert_str("简介", convert, false)?))
            .await?;
        writer.ln().await?;

        let mut buf = String::with_capacity(512);
        for line in introduction {
            buf.push_str(line);
            buf.push_str("\n\n");
        }
        // last '\n'
        buf.pop();

        writer.write(buf).await?;
        writer.flush().await?;
    }

    Ok(())
}

async fn generate_chapters<T>(novel: &Novel, base_path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let src_path = base_path.as_ref().join("src");
    let image_path = src_path.join("images");

    let mut volume_index = 1;
    let mut chapter_index = 1;
    let mut image_index = 1;

    for volume in &novel.volumes {
        let volume_path = src_path.join(format!("volume{}", utils::num_to_str(volume_index)));
        volume_index += 1;

        if !volume.chapters.is_empty() {
            let mut volume_writer = Writer::new(volume_path.join("README.md")).await?;
            volume_writer.writeln(format!("# {}", volume.title)).await?;
            volume_writer.flush().await?;

            for chapter in &volume.chapters {
                let chapter_path = volume_path
                    .join(format!("chapter{}", utils::num_to_str(chapter_index)))
                    .with_extension("md");
                chapter_index += 1;

                let mut chapter_writer = Writer::new(chapter_path).await?;

                chapter_writer
                    .writeln(format!("# {}", chapter.title))
                    .await?;
                chapter_writer.ln().await?;

                let mut buf = String::with_capacity(8192);
                for content in &chapter.contents {
                    match content {
                        Content::Text(line) => {
                            buf.push_str(line);
                            buf.push_str("\n\n");
                        }
                        Content::Image(image) => match super::new_image_name(image, image_index) {
                            Ok(image_name) => {
                                image_index += 1;

                                let image_path = image_path.join(image_name);
                                let image_path =
                                    pathdiff::diff_paths(image_path, &volume_path).unwrap();
                                let image_path_str =
                                    image_path.display().to_string().replace('\\', "/");

                                buf.push_str(&super::image_markdown_str(image_path_str));
                                buf.push_str("\n\n");
                            }
                            Err(err) => error!("Failed to get image name: {err}"),
                        },
                    }
                }

                // last '\n'
                buf.pop();

                chapter_writer.write(buf).await?;
                chapter_writer.flush().await?;
            }
        }
    }

    Ok(())
}
