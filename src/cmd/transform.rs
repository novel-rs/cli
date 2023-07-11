use std::{fs, path::PathBuf};

use clap::Args;
use color_eyre::eyre::Result;
use fluent_templates::Loader;
use novel_api::Timing;
use regex::Regex;
use tracing::info;

use crate::{
    cmd::Convert,
    utils::{self, UNIX_LINE_BREAK, WINDOWS_LINE_BREAK},
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "transform_command").unwrap())]
pub struct Transform {
    #[arg(help = LOCALES.lookup(&LANG_ID, "markdown_path").unwrap())]
    pub markdown_path: PathBuf,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").unwrap())]
    pub converts: Vec<Convert>,

    #[arg(short, long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "delete").unwrap())]
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

    let (mut meta_data, markdown) = utils::read_markdown(&input_markdown_file_path)?;

    meta_data.title = utils::convert_str(&meta_data.title, &config.converts)?;
    meta_data.author = utils::convert_str(&meta_data.author, &config.converts)?;
    meta_data.lang = utils::lang(&config.converts);
    if meta_data.description.is_some() {
        let mut description = Vec::new();

        for line in meta_data
            .description
            .as_ref()
            .unwrap()
            .split(UNIX_LINE_BREAK)
        {
            description.push(utils::convert_str(line, &config.converts)?);
        }

        meta_data.description = Some(description.join(UNIX_LINE_BREAK));
    }
    if meta_data.cover_image.is_some() {
        meta_data.cover_image = Some(PathBuf::from(
            utils::convert_image(
                input_dir.join(meta_data.cover_image.unwrap()),
                config.delete,
            )?
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
        ));
    }

    let events = utils::to_markdown_events(&markdown, &config.converts, &input_dir, config.delete)?;

    let mut buf = String::with_capacity(4096);
    buf.push_str(format!("---{}", UNIX_LINE_BREAK).as_str());
    buf.push_str(&serde_yaml::to_string(&meta_data)?);
    buf.push_str(format!("...{0}{0}", UNIX_LINE_BREAK).as_str());

    let mut markdown_buf = String::with_capacity(markdown.len());
    pulldown_cmark_to_cmark::cmark(events.iter(), &mut markdown_buf)?;

    let regex = Regex::new(&format!("({})+", UNIX_LINE_BREAK))?;
    buf.push_str(&regex.replace_all(&markdown_buf, format!("{0}{0}", UNIX_LINE_BREAK)));
    // \n
    buf.pop();

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
        utils::to_markdown_file_name(utils::convert_str(&meta_data.title, &config.converts)?);
    let output_markdown_file_path = input_dir.join(new_file_name);
    info!(
        "Output Markdown file path: `{}`",
        output_markdown_file_path.display()
    );

    if cfg!(windows) {
        buf = buf.replace(UNIX_LINE_BREAK, WINDOWS_LINE_BREAK);
    }
    fs::write(output_markdown_file_path, buf)?;

    info!("Time spent on `transform`: {}", timing.elapsed()?);

    Ok(())
}
