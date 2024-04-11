use std::{fmt, fs, path::PathBuf};

use color_eyre::eyre::Result;
use novel_api::Timing;
use pulldown_cmark::{Event, MetadataBlockKind, Tag, TagEnd};
use tracing::{error, info, warn};

use crate::{
    cmd::Convert,
    utils::{self, Content, Metadata, Novel},
};

pub fn generate_pandoc_markdown<T>(novel: Novel, convert: T) -> Result<()>
where
    T: AsRef<[Convert]>,
{
    let mut timing = Timing::new();

    assert!(!novel.name.is_empty(), "The novel name is empty");

    let output_dir_path = utils::to_novel_dir_name(&novel.name);
    if output_dir_path.is_dir() {
        warn!("The Pandoc output directory already exists and will be deleted");
        utils::remove_file_or_dir(&output_dir_path)?;
    }

    fs::create_dir(&output_dir_path)?;
    let output_dir_path = dunce::canonicalize(output_dir_path)?;

    let markdown_file_path = output_dir_path.join(utils::to_markdown_file_name(&novel.name));

    let mut buf = String::with_capacity(2 * 1024 * 1024);

    write_metadata(&novel, &mut buf, &convert)?;
    write_introduction(&novel, &mut buf, &convert)?;
    write_chapters(&novel, &mut buf)?;

    // last \n
    if buf.ends_with("\n\n") {
        buf.pop();
    }

    fs::write(markdown_file_path, &buf)?;
    super::save_image(&novel, output_dir_path)?;

    info!(
        "Time spent on `generate pandoc markdown`: {}",
        timing.elapsed()?
    );

    Ok(())
}

fn write_metadata<T, C>(novel: &Novel, mut buf: T, convert: C) -> Result<()>
where
    T: fmt::Write,
    C: AsRef<[Convert]>,
{
    let description = novel
        .introduction
        .as_ref()
        .map(|introduction| introduction.join("\n"));

    let cover_image = if let Some(ref cover_image) = novel.cover_image {
        match super::cover_image_name(cover_image) {
            Ok(cover_image_name) => Some(PathBuf::from(cover_image_name)),
            Err(err) => {
                error!("Failed to get cover image name: {err}");
                None
            }
        }
    } else {
        None
    };

    let metadata = Metadata {
        title: novel.name.clone(),
        author: novel.author_name.clone(),
        lang: utils::lang(convert),
        description,
        cover_image,
    };

    let metadata_block = vec![
        Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)),
        Event::Text(serde_yaml::to_string(&metadata)?.into()),
        Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)),
    ];

    pulldown_cmark_to_cmark::cmark(metadata_block.iter(), &mut buf)?;
    buf.write_char('\n')?;

    Ok(())
}

fn write_introduction<T, C>(novel: &Novel, mut buf: T, convert: C) -> Result<()>
where
    T: fmt::Write,
    C: AsRef<[Convert]>,
{
    if let Some(ref introduction) = novel.introduction {
        buf.write_str(&format!(
            "# {}\n\n",
            utils::convert_str("简介", convert, false)?
        ))?;

        for line in introduction {
            buf.write_str(line)?;
            buf.write_str("\n\n")?;
        }
    }

    Ok(())
}

fn write_chapters<T>(novel: &Novel, mut buf: T) -> Result<()>
where
    T: fmt::Write,
{
    let mut image_index = 1;

    for volume in &novel.volumes {
        if !volume.chapters.is_empty() {
            buf.write_str(&format!("# {}\n\n", volume.title))?;

            for chapter in &volume.chapters {
                buf.write_str(&format!("## {}\n\n", chapter.title))?;

                for content in &chapter.contents {
                    match content {
                        Content::Text(line) => {
                            buf.write_str(line)?;
                            buf.write_str("\n\n")?;
                        }
                        Content::Image(image) => match super::new_image_name(image, image_index) {
                            Ok(image_name) => {
                                image_index += 1;

                                buf.write_str(&super::image_markdown_str(image_name))?;
                                buf.write_str("\n\n")?;
                            }
                            Err(err) => error!("Failed to get image name: {err}"),
                        },
                    }
                }
            }
        }
    }

    Ok(())
}
