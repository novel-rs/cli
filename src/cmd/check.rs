use std::{path::PathBuf, sync::RwLock};

use clap::Args;
use color_eyre::eyre::{ensure, Result};
use fluent_templates::Loader;
use hashbrown::HashSet;
use novel_api::Timing;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};
use rayon::prelude::*;
use tracing::debug;

use crate::{
    utils::{self, CurrentDir},
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "check_command").unwrap())]
pub struct Check {
    #[arg(help = LOCALES.lookup(&LANG_ID, "markdown_path").unwrap())]
    pub markdown_path: PathBuf,
}

pub fn execute(config: Check) -> Result<()> {
    let mut timing = Timing::new();

    let (meta_data, markdown) = utils::read_markdown(&config.markdown_path)?;

    let markdown_path = dunce::canonicalize(&config.markdown_path)?;
    let current_dir = CurrentDir::new(markdown_path.parent().unwrap())?;

    ensure!(
        meta_data.lang_is_ok(),
        "The lang field must be zh-Hant or zh-Hans: `{}`",
        meta_data.lang
    );
    ensure!(
        meta_data.cover_image_is_ok(),
        "Cover image does not exist: `{}`",
        meta_data.cover_image.unwrap().display()
    );

    let parser = Parser::new_ext(&markdown, Options::empty());
    let events = parser.into_offset_iter().collect::<Vec<(_, _)>>();
    let char_set = RwLock::new(HashSet::new());

    // TODO i18n output
    events
        .into_par_iter()
        .for_each(|(event, range)| match event {
            Event::Start(tag) => match tag {
                Tag::Heading(heading_level, _, _) => {
                    let title = markdown[range].trim_start_matches('#').trim();

                    if heading_level == HeadingLevel::H1 && !check_volume_title(title) {
                        println(format!("Irregular volume title format: `{title}`"));
                    } else if heading_level == HeadingLevel::H2 && !check_chapter_title(title) {
                        println(format!("Irregular chapter title format: `{title}`"));
                    }
                }
                Tag::Image(_, path, _) => {
                    let image_path = PathBuf::from(&path.to_string());

                    if !image_path.is_file() {
                        println(format!("Image `{}` does not exist", image_path.display()));
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
                | Tag::Link(_, _, _) => {
                    let content = &markdown[range].trim();

                    println(format!(
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
                        if char_set.read().unwrap().contains(&c) {
                            continue;
                        } else {
                            char_set.write().unwrap().insert(c);

                            println(format!(
                                "Irregular char: `{}`, at `{}`",
                                c,
                                markdown[range.clone()].trim()
                            ));
                        }
                    }
                }
            }
            Event::End(_) => (),
            Event::HardBreak => (),
            Event::Code(_)
            | Event::Html(_)
            | Event::FootnoteReference(_)
            | Event::SoftBreak
            | Event::Rule
            | Event::TaskListMarker(_) => {
                let content = &markdown[range].trim();

                println(format!(
                    "Markdown event that should not appear: `{event:?}`, content: `{content}`"
                ));
            }
        });

    current_dir.restore()?;

    debug!("Time spent on `check`: {}", timing.elapsed()?);

    Ok(())
}

#[inline]
fn println(msg: String) {
    println!("{} {}", utils::emoji("⚠️"), msg);
}

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

#[must_use]
#[inline]
fn check_chapter_title<T>(title: T) -> bool
where
    T: AsRef<str>,
{
    let regex = regex!(r"第([零一二三四五六七八九十百千]|[0-9]){1,7}[章话] .+");
    regex.is_match(title.as_ref())
}

#[must_use]
#[inline]
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
