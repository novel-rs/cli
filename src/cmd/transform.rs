use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use color_eyre::eyre::{bail, Result};
use fluent_templates::Loader;
use novel_api::Timing;
use pulldown_cmark::{Event, MetadataBlockKind, Options, Tag, TagEnd, TextMergeWithOffset};
use tracing::{debug, info};
use walkdir::WalkDir;

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
    #[arg(help = LOCALES.lookup(&LANG_ID, "file_path"))]
    pub file_path: PathBuf,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts"))]
    pub converts: Vec<Convert>,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "delete"))]
    pub delete: bool,
}

pub fn execute(config: Transform) -> Result<()> {
    let mut timing = Timing::new();

    let input_file_path;
    let input_file_parent_path;

    if utils::is_markdown_or_txt_file(&config.file_path)? {
        input_file_path = dunce::canonicalize(&config.file_path)?;
        input_file_parent_path = input_file_path.parent().unwrap().to_path_buf();
    } else if let Ok(Some(path)) =
        utils::try_get_markdown_or_txt_file_name_in_dir(&config.file_path)
    {
        input_file_path = path;
        input_file_parent_path = dunce::canonicalize(&config.file_path)?;
    } else {
        bail!("Invalid input path: `{}`", config.file_path.display());
    }
    info!("Input file path: `{}`", input_file_path.display());

    let input_file_stem = input_file_path.file_stem().unwrap().to_str().unwrap();
    let input_file_ext = input_file_path.extension().unwrap().to_str().unwrap();

    let bytes = fs::read(&input_file_path)?;
    let markdown = simdutf8::basic::from_utf8(&bytes)?;
    let mut parser =
        TextMergeWithOffset::new_ext(markdown, Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    let mut metadata = utils::get_metadata(&mut parser)?;
    convert_metadata(&mut metadata, &config.converts, &input_file_parent_path)?;

    let mut image_index = 1;
    let mut in_heading = false;
    let parser = parser.map(|(event, _)| match event {
        Event::Start(Tag::Heading {
            level,
            id,
            classes,
            attrs,
        }) => {
            in_heading = true;
            Event::Start(Tag::Heading {
                level,
                id,
                classes,
                attrs,
            })
        }
        Event::End(TagEnd::Heading(level)) => {
            in_heading = false;
            Event::End(TagEnd::Heading(level))
        }
        Event::Text(text) => Event::Text(
            utils::convert_str(text, &config.converts, in_heading)
                .unwrap()
                .into(),
        ),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let new_image_path =
                utils::convert_image_ext(input_file_parent_path.join(dest_url.as_ref())).unwrap();

            let new_image_path =
                utils::convert_image_file_stem(new_image_path, utils::num_to_str(image_index))
                    .unwrap();
            image_index += 1;

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

    let mut buf = String::with_capacity(markdown.len());
    pulldown_cmark_to_cmark::cmark(metadata_block.iter(), &mut buf)?;
    buf.write_char('\n')?;
    pulldown_cmark_to_cmark::cmark(parser, &mut buf)?;
    buf.write_char('\n')?;

    if config.delete {
        utils::remove_file_or_dir(&input_file_path)?;
    } else {
        let backup_file_path =
            input_file_parent_path.join(format!("{input_file_stem}.old.{input_file_ext}"));
        info!("Backup file path: `{}`", backup_file_path.display());

        fs::rename(&input_file_path, backup_file_path)?;
    }

    let new_file_name = utils::to_novel_dir_name(utils::convert_str(
        &metadata.title,
        &config.converts,
        false,
    )?)
    .with_extension(input_file_ext);
    let output_file_path = input_file_parent_path.join(new_file_name);
    info!("Output file path: `{}`", output_file_path.display());

    if cfg!(windows) {
        buf = buf.replace('\n', "\r\n");
    }
    fs::write(&output_file_path, buf)?;

    if config.delete {
        let image_paths = utils::read_markdown_to_images(&output_file_path)?;

        let mut to_remove = Vec::new();
        for entry in WalkDir::new(&input_file_parent_path).max_depth(1) {
            let path = entry?.path().to_path_buf();

            if path != output_file_path && path != input_file_parent_path {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if !image_paths.contains(&PathBuf::from(file_name)) {
                    to_remove.push(path);
                }
            }
        }

        utils::remove_file_or_dir_all(&to_remove)?;
    }

    debug!("Time spent on `transform`: {}", timing.elapsed()?);

    Ok(())
}

fn convert_metadata(metadata: &mut Metadata, converts: &[Convert], input_dir: &Path) -> Result<()> {
    metadata.title = utils::convert_str(&metadata.title, converts, false)?;
    metadata.author = utils::convert_str(&metadata.author, converts, false)?;
    metadata.lang = utils::lang(converts);

    if metadata.description.is_some() {
        let mut description = Vec::with_capacity(4);

        for line in metadata.description.as_ref().unwrap().split('\n') {
            description.push(utils::convert_str(line, converts, false).unwrap());
        }

        metadata.description = Some(description.join("\n"));
    }

    if metadata.cover_image.is_some() {
        let new_image_path =
            utils::convert_image_ext(input_dir.join(metadata.cover_image.as_ref().unwrap()))
                .unwrap();

        let new_image_path = utils::convert_image_file_stem(new_image_path, "cover").unwrap();

        metadata.cover_image = Some(PathBuf::from(
            new_image_path.file_name().unwrap().to_str().unwrap(),
        ));
    }

    Ok(())
}
