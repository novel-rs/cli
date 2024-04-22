use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use color_eyre::eyre::{bail, ensure, Result};
use fluent_templates::Loader;
use hashbrown::HashSet;
use novel_api::Timing;
use pulldown_cmark::{Event, HeadingLevel, Options, Tag, TextMergeWithOffset};
use tracing::{debug, info};

use crate::{
    utils::{self, CurrentDir, Lang},
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "check_command"))]
pub struct Check {
    #[arg(help = LOCALES.lookup(&LANG_ID, "file_path"))]
    pub file_path: PathBuf,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "basic_check"))]
    pub basic_check: bool,
}

pub fn execute(config: Check) -> Result<()> {
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

    let current_dir = CurrentDir::new(input_file_parent_path)?;

    let bytes = fs::read(&input_file_path)?;
    let markdown = simdutf8::basic::from_utf8(&bytes)?;
    let mut parser =
        TextMergeWithOffset::new_ext(markdown, Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    let lang = check_metadata(&mut parser)?;

    let max_width = (utils::terminal_size().0 / 2) as usize;
    let mut char_set = HashSet::new();
    parser.for_each(|(event, range)| match event {
        Event::Start(tag) => match tag {
            Tag::Heading { level, .. } => {
                let title = markdown[range].trim_start_matches('#').trim();

                if level == HeadingLevel::H1 {
                    if !check_volume_title(title, lang) {
                        println_msg(format!("Irregular volume title format: `{title}`"));
                    }
                } else if level == HeadingLevel::H2 {
                    if !check_chapter_title(title, lang) {
                        println_msg(format!("Irregular chapter title format: `{title}`"));
                    }
                } else {
                    println_msg(format!(
                        "Irregular heading level: `{level:?}`, content: `{title}`"
                    ));
                }
            }
            Tag::Image { dest_url, .. } => {
                let image_path = Path::new(dest_url.as_ref());

                if !image_path.is_file() {
                    println_msg(format!("Image `{}` does not exist", image_path.display()));
                }
            }
            Tag::Paragraph => (),
            Tag::BlockQuote
            | Tag::CodeBlock(_)
            | Tag::List(_)
            | Tag::Item
            | Tag::FootnoteDefinition(_)
            | Tag::Table(_)
            | Tag::TableHead
            | Tag::TableRow
            | Tag::TableCell
            | Tag::Emphasis
            | Tag::Strong
            | Tag::Strikethrough
            | Tag::Link { .. }
            | Tag::HtmlBlock
            | Tag::MetadataBlock(_) => {
                if !config.basic_check {
                    let content = console::truncate_str(markdown[range].trim(), max_width, "...");

                    println_msg(format!(
                        "Markdown tag that should not appear: `{tag:?}`, content: `{content}`"
                    ));
                }
            }
        },
        Event::Text(text) => {
            if !config.basic_check {
                for c in text.chars() {
                    if !utils::is_cjk(c)
                        && !utils::is_punctuation(c)
                        && !c.is_ascii_alphanumeric()
                        && c != ' '
                    {
                        if char_set.contains(&c) {
                            continue;
                        } else {
                            char_set.insert(c);

                            println_msg(format!(
                                "Irregular char: `{}`, at `{}`",
                                c,
                                console::truncate_str(
                                    markdown[range.clone()].trim(),
                                    max_width,
                                    "..."
                                )
                            ));
                        }
                    }
                }
            }
        }
        Event::End(_) => (),
        Event::HardBreak
        | Event::Code(_)
        | Event::Html(_)
        | Event::FootnoteReference(_)
        | Event::SoftBreak
        | Event::Rule
        | Event::TaskListMarker(_)
        | Event::InlineHtml(_) => {
            if !config.basic_check {
                let content = console::truncate_str(markdown[range].trim(), max_width, "...");

                println_msg(format!(
                    "Markdown event that should not appear: `{event:?}`, content: `{content}`"
                ));
            }
        }
    });

    current_dir.restore()?;

    debug!("Time spent on `check`: {}", timing.elapsed()?);

    Ok(())
}

fn check_metadata(parser: &mut TextMergeWithOffset) -> Result<Lang> {
    let metadata = utils::get_metadata(parser)?;

    ensure!(
        metadata.cover_image_is_ok(),
        "Cover image does not exist: `{}`",
        metadata.cover_image.unwrap().display()
    );

    Ok(metadata.lang)
}

fn println_msg(msg: String) {
    println!("{} {}", utils::emoji("⚠️"), msg);
}

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

#[must_use]
fn check_chapter_title<T>(title: T, lang: Lang) -> bool
where
    T: AsRef<str>,
{
    let title = title.as_ref();

    match lang {
        Lang::ZhHant => {
            let regex = regex!(r"第([零一二三四五六七八九十百千]|[0-9]){1,7}[章話] .+");
            regex.is_match(title.as_ref())
        }
        Lang::ZhHans => {
            let regex = regex!(r"第([零一二三四五六七八九十百千]|[0-9]){1,7}[章话] .+");
            regex.is_match(title.as_ref())
        }
    }
}

#[must_use]
fn check_volume_title<T>(title: T, lang: Lang) -> bool
where
    T: AsRef<str>,
{
    let title = title.as_ref();

    match lang {
        Lang::ZhHant => {
            let regex = regex!(r"第([一二三四五六七八九十]|[0-9]){1,3}卷 .+");
            regex.is_match(title) || title == "簡介"
        }
        Lang::ZhHans => {
            let regex = regex!(r"第([一二三四五六七八九十]|[0-9]){1,3}卷 .+");
            regex.is_match(title) || title == "简介"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_chapter_title_test() {
        assert!(check_chapter_title("第一章 被俘虏的开始", Lang::ZhHans));
        assert!(check_chapter_title(
            "第一百三十二章 标标标标标标标标标",
            Lang::ZhHans
        ));
        assert!(check_chapter_title("第123章 标题标标标标", Lang::ZhHans));
        assert!(!check_chapter_title("第一章 ", Lang::ZhHans));
        assert!(!check_chapter_title("第1二3话", Lang::ZhHans));
        assert!(!check_chapter_title("第123话标题", Lang::ZhHans));
        assert!(!check_chapter_title("123话 标题", Lang::ZhHans));
    }

    #[test]
    fn check_volume_title_test() {
        assert!(check_volume_title(
            "第三十二卷 标标标标标标标标标",
            Lang::ZhHans
        ));
        assert!(!check_volume_title("第123话 标题标标标标", Lang::ZhHans));
        assert!(!check_volume_title("第1卷 ", Lang::ZhHans));
    }
}
