use std::{fs, path::PathBuf};

use ahash::AHashSet;
use anyhow::{ensure, Result};
use clap::Args;
use console::{Alignment, Emoji};
use fluent_templates::Loader;
use parking_lot::RwLock;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};
use rayon::prelude::*;

use crate::{utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Debug, Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "check_command").expect("`check_command` does not exists"))]
pub struct Check {
    #[arg(help = LOCALES.lookup(&LANG_ID, "markdown_path").expect("`markdown_path` does not exists"))]
    pub markdown_path: PathBuf,
}

pub fn execute(config: Check) -> Result<()> {
    ensure!(
        utils::is_markdown(&config.markdown_path),
        "The input file is not in markdown format"
    );

    let bytes = fs::read(&config.markdown_path)?;
    let markdown = simdutf8::basic::from_utf8(&bytes)?;

    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    let parser = Parser::new_ext(markdown, options);

    let events = parser.into_offset_iter().collect::<Vec<(_, _)>>();
    let char_set = RwLock::new(AHashSet::new());

    // TODO 检查图片是否存在
    events.into_par_iter().for_each(|(event, range)| {
        if let Event::Start(Tag::Heading(heading_level, _, _)) = &event {
            let title = markdown[range].trim_start_matches('#').trim();

            if *heading_level == HeadingLevel::H1 && !check_volume_title(title) {
                println!("{} Irregular volume title format: {title}", emoji());
            } else if *heading_level == HeadingLevel::H2 && !check_chapter_title(title) {
                println!("{} Irregular chapter title format: {title}", emoji());
            }
        } else if let Event::Text(text) = &event {
            for c in text.chars() {
                if !utils::is_cjk(c)
                    && !utils::is_punctuation(c)
                    && !c.is_ascii_alphanumeric()
                    && c != ' '
                {
                    if char_set.read().contains(&c) {
                        continue;
                    } else {
                        char_set.write().insert(c);
                        println!(
                            "{} Irregular char: {}, at {}",
                            emoji(),
                            c,
                            markdown[range.clone()].trim()
                        );
                    }
                }
            }
        }
    });

    Ok(())
}

#[must_use]
fn emoji() -> String {
    let emoji = Emoji("⚠️", ">").to_string();
    let emoji = console::pad_str(&emoji, 2, Alignment::Left, None);
    emoji.to_string()
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
