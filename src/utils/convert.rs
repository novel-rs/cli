use color_eyre::eyre::Result;
use novel_api::Timing;
use once_cell::sync::OnceCell;
use opencc_rs::{Config, OpenCC};
use tracing::info;

use crate::cmd::Convert;

use super::{Content, Novel};

pub async fn convert<T>(novel: &mut Novel, converts: T) -> Result<()>
where
    T: AsRef<[Convert]>,
{
    if converts.as_ref().is_empty() {
        return Ok(());
    }

    let mut timing = Timing::new();

    novel.name = convert_str(&novel.name, &converts)?;

    novel.author_name = convert_str(&novel.author_name, &converts)?;

    if novel.introduction.is_some() {
        for line in novel.introduction.as_mut().unwrap() {
            *line = convert_str(&line, &converts)?;
        }
    }

    for volume in &mut novel.volumes {
        volume.title = convert_str(&volume.title, &converts)?;

        for chapter in &mut volume.chapters {
            chapter.title = convert_str(&chapter.title, &converts)?;

            for content in chapter.contents.write().await.iter_mut() {
                if let Content::Text(line) = content {
                    *line = convert_str(&line, &converts)?;
                }
            }
        }
    }

    info!("Time spent on `convert`: {}", timing.elapsed()?);

    Ok(())
}

pub fn convert_str<T, E>(str: T, converts: E) -> Result<String>
where
    T: AsRef<str>,
    E: AsRef<[Convert]>,
{
    let converts = converts.as_ref();

    if converts.is_empty() {
        return Ok(str.as_ref().to_string());
    } else {
        let mut result = String::new();

        static OPENCC_S2T: OnceCell<OpenCC> = OnceCell::new();
        static OPENCC_T2S: OnceCell<OpenCC> = OnceCell::new();
        static OPENCC_JP2T2S: OnceCell<OpenCC> = OnceCell::new();

        if converts.contains(&Convert::JP2T2S) {
            result = OPENCC_JP2T2S
                .get_or_try_init(|| OpenCC::new(vec![Config::JP2T, Config::TW2S]))?
                .convert(&str)?;
        } else if converts.contains(&Convert::T2S) {
            result = OPENCC_T2S
                .get_or_try_init(|| OpenCC::new(vec![Config::TW2S]))?
                .convert(&str)?;
        } else if converts.contains(&Convert::S2T) {
            result = OPENCC_S2T
                .get_or_try_init(|| OpenCC::new(vec![Config::S2T]))?
                .convert(&str)?;
        }

        if converts.contains(&Convert::CUSTOM) {
            if result.is_empty() {
                result = custom_convert(str);
            } else {
                result = custom_convert(result);
            }

            if converts.contains(&Convert::JP2T2S) || converts.contains(&Convert::T2S) {
                let mut new_result = String::with_capacity(result.len());
                for c in result.chars() {
                    match super::CONVERT_T2S_MAP.get(&c) {
                        Some(new) => {
                            new_result.push(*new);
                        }
                        None => new_result.push(c),
                    }
                }

                result = new_result;
            }
        }

        Ok(result.trim().to_string())
    }
}

#[must_use]
fn custom_convert<T>(str: T) -> String
where
    T: AsRef<str>,
{
    if str.as_ref().is_empty() {
        return String::default();
    }

    let mut s = String::new();
    for c in html_escape::decode_html_entities(str.as_ref())
        .to_string()
        .chars()
    {
        match super::CONVERT_MAP.get(&c) {
            Some(new) => {
                s.push(*new);
            }
            None => s.push(c),
        }
    }

    let mut result = String::new();
    for (c, next_c) in s.chars().zip(s.chars().skip(1)) {
        do_custom_convert(c, Some(next_c), &mut result);
    }
    do_custom_convert(s.chars().last().unwrap(), None, &mut result);

    result
}

fn do_custom_convert(c: char, next_c: Option<char>, result: &mut String) {
    let space = ' ';
    let last = result.chars().last();

    if
    // https://en.wikipedia.org/wiki/Zero-width_space
    c == '\u{200B}'
        // https://en.wikipedia.org/wiki/Zero-width_non-joiner
        || c == '\u{200C}'
        // https://en.wikipedia.org/wiki/Zero-width_joiner
        || c == '\u{200D}'
        // https://en.wikipedia.org/wiki/Word_joiner
        || c == '\u{2060}'
        // https://en.wikipedia.org/wiki/Byte_order_mark
        || c == '\u{FEFF}'
        || c.is_control()
    {
        // do nothing
    } else if c.is_whitespace() {
        if last.is_some_and(|c| !super::is_punctuation(c)) {
            result.push(space)
        }
    } else if super::is_punctuation(c) {
        if last.is_some_and(|c| c.is_whitespace()) {
            result.pop();
        }

        if c == '?' {
            result.push('？');
        } else if c == '!' {
            result.push('！');
        } else if c == ',' {
            result.push('，');
        } else if c == ':' {
            // e.g. 08:00
            if last.is_some_and(|c| c.is_ascii_digit())
                && next_c.is_some_and(|c| c.is_ascii_digit())
            {
                result.push(':');
            } else {
                result.push('：');
            }
        } else if c == ';' {
            // https://en.wikipedia.org/wiki/Non-breaking_space
            if result.ends_with("&nbsp") {
                result.truncate(result.len() - 5);
                result.push(' ');
            } else if result.ends_with("&lt") {
                result.truncate(result.len() - 3);
                result.push('<');
            } else if result.ends_with("&gt") {
                result.truncate(result.len() - 3);
                result.push('>');
            } else if result.ends_with("&quot") {
                result.truncate(result.len() - 5);
                result.push('"');
            } else if result.ends_with("&apos") {
                result.truncate(result.len() - 5);
                result.push('\'');
            } else if result.ends_with("&amp") {
                result.truncate(result.len() - 4);
                result.push('&');
            } else {
                result.push('；');
            }
        } else if c == '(' {
            result.push('（');
        } else if c == ')' {
            result.push('）');
        } else if c == '。' || c == '，' || c == '、' {
            if last.is_some_and(|last_char| last_char == c) {
                // do nothing
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    } else {
        result.push(c);
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn convert() -> Result<()> {
        let config = vec![Convert::JP2T2S, Convert::CUSTOM];

        assert_eq!(convert_str("顛覆", &config)?, "颠覆");
        assert_eq!(convert_str("幺", &config)?, "幺");
        assert_eq!(convert_str("妳", &config)?, "你");
        assert_eq!(convert_str("Ｑ０", &config)?, "Q0");
        assert_eq!(convert_str("“安装后”", &config)?, "“安装后”");
        assert_eq!(convert_str("&amp;", &config)?, "&");
        assert_eq!(convert_str("安裝後?", &config)?, "安装后？");
        assert_eq!(convert_str("，，，", &config)?, "，");
        assert_eq!(convert_str("安　装", &config)?, "安 装");
        assert_eq!(convert_str("你\n好", &config)?, "你好");
        assert_eq!(convert_str("08:00", &config)?, "08:00");
        assert_eq!(convert_str("接著", &config)?, "接着");

        Ok(())
    }
}
