use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use color_eyre::eyre::{ensure, Result};
use fluent_templates::Loader;
use hashbrown::HashSet;
use novel_api::Timing;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};
use tracing::debug;

use crate::{
    utils::{self, CurrentDir},
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "check_command"))]
pub struct Check {
    #[arg(help = LOCALES.lookup(&LANG_ID, "markdown_path"))]
    pub markdown_path: PathBuf,
}

pub fn execute(config: Check) -> Result<()> {
    let mut timing = Timing::new();

    utils::ensure_markdown_or_txt_file(&config.markdown_path)?;

    let markdown_path = dunce::canonicalize(&config.markdown_path)?;
    let current_dir = CurrentDir::new(markdown_path.parent().unwrap())?;

    let bytes = fs::read(&markdown_path)?;
    let markdown = simdutf8::basic::from_utf8(&bytes)?;
    let mut parser = Parser::new_ext(markdown, Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    check_metadata(&mut parser)?;

    let mut char_set = HashSet::new();
    // TODO i18n output
    parser
        .into_offset_iter()
        .for_each(|(event, range)| match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    let title = markdown[range].trim_start_matches('#').trim();

                    if level == HeadingLevel::H1 {
                        if !check_volume_title(title) {
                            println_msg(format!("Irregular volume title format: `{title}`"));
                        }
                    } else if level == HeadingLevel::H2 {
                        if !check_chapter_title(title) {
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
                    let content = markdown[range].trim();

                    println_msg(format!(
                        "Markdown tag that should not appear: `{tag:?}`, content: `{content}`"
                    ));
                }
            },
            Event::Text(text) => {
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
                                &markdown[range.clone()]
                            ));
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
                let content = markdown[range].trim();

                println_msg(format!(
                    "Markdown event that should not appear: `{event:?}`, content: `{content}`"
                ));
            }
        });

    current_dir.restore()?;

    debug!("Time spent on `check`: {}", timing.elapsed()?);

    Ok(())
}

fn check_metadata(parser: &mut Parser) -> Result<()> {
    let metadata = utils::get_metadata(parser)?;

    ensure!(
        metadata.lang_is_ok(),
        "The lang field must be zh-Hant or zh-Hans: `{}`",
        metadata.lang
    );
    ensure!(
        metadata.cover_image_is_ok(),
        "Cover image does not exist: `{}`",
        metadata.cover_image.unwrap().display()
    );

    Ok(())
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
fn check_chapter_title<T>(title: T) -> bool
where
    T: AsRef<str>,
{
    let regex = regex!(r"第([零一二三四五六七八九十百千]|[0-9]){1,7}[章话] .+");
    regex.is_match(title.as_ref())
}

#[must_use]
fn check_volume_title<T>(title: T) -> bool
where
    T: AsRef<str>,
{
    let regex = regex!(r"第([一二三四五六七八九十]|[0-9]){1,3}卷 .+");
    regex.is_match(title.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_chapter_title_test() {
        assert!(check_chapter_title("第一章 被俘虏的开始"));
        assert!(check_chapter_title("第一百三十二章 标标标标标标标标标"));
        assert!(check_chapter_title("第123章 标题标标标标"));
        assert!(!check_chapter_title("第一章 "));
        assert!(!check_chapter_title("第1二3话"));
        assert!(!check_chapter_title("第123话标题"));
        assert!(!check_chapter_title("123话 标题"));
    }

    #[test]
    fn check_volume_title_test() {
        assert!(check_volume_title("第三十二卷 标标标标标标标标标"));
        assert!(!check_volume_title("第123话 标题标标标标"));
        assert!(!check_volume_title("第1卷 "));
    }
}
