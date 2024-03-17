use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use color_eyre::eyre::Result;
use fluent_templates::Loader;
use novel_api::Timing;
use pulldown_cmark::{Event, MetadataBlockKind, Options, Tag, TagEnd, TextMergeWithOffset};
use tracing::{debug, info};

use crate::{
    cmd::Convert,
    utils::{self, Metadata},
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "transform_command"))]
pub struct Transform {
    #[arg(help = LOCALES.lookup(&LANG_ID, "markdown_path"))]
    pub markdown_path: PathBuf,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts"))]
    pub converts: Vec<Convert>,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "delete"))]
    pub delete: bool,
}

pub fn execute(config: Transform) -> Result<()> {
    let mut timing = Timing::new();

    utils::ensure_markdown_file(&config.markdown_path)?;

    let input_markdown_file_path = dunce::canonicalize(&config.markdown_path)?;
    let input_dir = input_markdown_file_path.parent().unwrap().to_path_buf();
    let input_file_stem = input_markdown_file_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    info!(
        "Input Markdown file path: `{}`",
        input_markdown_file_path.display()
    );

    let bytes = fs::read(&input_markdown_file_path)?;
    let markdown = simdutf8::basic::from_utf8(&bytes)?;
    let mut parser =
        TextMergeWithOffset::new_ext(markdown, Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    let mut metadata = utils::get_metadata(&mut parser)?;
    convert_metadata(&mut metadata, &config.converts, &input_dir, config.delete)?;

    let parser = parser.map(|(event, _)| match event {
        Event::Text(text) => {
            Event::Text(utils::convert_str(text, &config.converts).unwrap().into())
        }
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let new_image_path =
                utils::convert_image(input_dir.join(dest_url.as_ref()), config.delete).unwrap();

            Event::Start(Tag::Image {
                link_type,
                dest_url: new_image_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .into(),
                title,
                id,
            })
        }
        _ => event,
    });

    let metadata_block = vec![
        Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)),
        Event::Text(serde_yaml::to_string(&metadata)?.into()),
        Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)),
    ];

    let mut markdown_buf = String::with_capacity(markdown.len());
    pulldown_cmark_to_cmark::cmark(metadata_block.iter(), &mut markdown_buf)?;
    markdown_buf.write_char('\n')?;
    pulldown_cmark_to_cmark::cmark(parser, &mut markdown_buf)?;
    markdown_buf.write_char('\n')?;

    if config.delete {
        utils::remove_file_or_dir(input_markdown_file_path)?;
    } else {
        let backup_markdown_file_path = input_dir.join(format!("{input_file_stem}.old.md"));
        info!(
            "Backup Markdown file path: `{}`",
            backup_markdown_file_path.display()
        );

        fs::rename(&input_markdown_file_path, backup_markdown_file_path)?;
    }

    let new_file_name =
        utils::to_markdown_file_name(utils::convert_str(&metadata.title, &config.converts)?);
    let output_markdown_file_path = input_dir.join(new_file_name);
    info!(
        "Output Markdown file path: `{}`",
        output_markdown_file_path.display()
    );

    if cfg!(windows) {
        markdown_buf = markdown_buf.replace('\n', "\r\n");
    }
    fs::write(output_markdown_file_path, markdown_buf)?;

    debug!("Time spent on `transform`: {}", timing.elapsed()?);

    Ok(())
}

fn convert_metadata(
    metadata: &mut Metadata,
    converts: &[Convert],
    input_dir: &Path,
    delete: bool,
) -> Result<()> {
    metadata.title = utils::convert_str(&metadata.title, converts)?;
    metadata.author = utils::convert_str(&metadata.author, converts)?;
    metadata.lang = utils::lang(converts);

    if metadata.description.is_some() {
        let mut description = Vec::with_capacity(4);

        for line in metadata.description.as_ref().unwrap().split('\n') {
            description.push(utils::convert_str(line, converts).unwrap());
        }

        metadata.description = Some(description.join("\n"));
    }

    if metadata.cover_image.is_some() {
        metadata.cover_image = Some(PathBuf::from(
            utils::convert_image(
                input_dir.join(metadata.cover_image.as_ref().unwrap()),
                delete,
            )?
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
        ));
    }

    Ok(())
}
