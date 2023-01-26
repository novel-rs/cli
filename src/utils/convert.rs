use anyhow::{Ok, Result};
use novel_api::Timing;
use once_cell::sync::OnceCell;
use opencc_rs::{Config, OpenCC};
use tracing::info;

use crate::cmd::Convert;

use super::{Content, Novel};

pub async fn convert(novel: &mut Novel, converts: &Vec<Convert>) -> Result<()> {
    if converts.is_empty() {
        return Ok(());
    }

    let mut timing = Timing::new();

    novel.name = convert_str(&novel.name, converts)?;

    if novel.introduction.is_some() {
        for line in novel.introduction.as_mut().unwrap() {
            *line = convert_str(&line, converts)?;
        }
    }

    for volume in &mut novel.volumes {
        volume.title = convert_str(&volume.title, converts)?;

        for chapter in &mut volume.chapters {
            chapter.title = convert_str(&chapter.title, converts)?;

            for content in chapter.contents.write().await.iter_mut() {
                if let Content::Text(line) = content {
                    *line = convert_str(&line, converts)?;
                }
            }
        }
    }

    info!("Time spent on `convert`: {}", timing.elapsed()?);

    Ok(())
}

fn do_custom_convert(c: char, next_c: Option<char>, result: &mut String) {
    let space = ' ';
    let last = result.chars().last();

    // https://en.wikipedia.org/wiki/Zero-width_space
    if c == '\n' {
        result.push(c);
    } else if c == '\u{200B}'
        // https://en.wikipedia.org/wiki/Zero-width_non-joiner
        || c == '\u{200C}'
        // https://en.wikipedia.org/wiki/Zero-width_joiner
        || c == '\u{200D}'
        // https://en.wikipedia.org/wiki/Word_joiner
        || c == '\u{2060}'
        || c == '\u{FEFF}'
        || c.is_control()
    {
        // nothing
    } else if c.is_whitespace() {
        if novel_api::is_some_and(last, |c| !super::is_chinese_punctuation(c)) {
            result.push(space)
        }
    } else if super::is_chinese_punctuation(c) || super::is_english_punctuation(c) {
        if novel_api::is_some_and(last, |c| c == space) {
            result.pop();
        }

        if c == '?' {
            result.push('？');
        } else if c == '!' {
            result.push('！');
        } else if c == ',' {
            result.push('，');
        } else if c == ':' {
            // 08:00
            if novel_api::is_some_and(last, |c| c.is_ascii_digit())
                && novel_api::is_some_and(next_c, |c| c.is_ascii_digit())
            {
                result.push(':');
            } else {
                result.push('：');
            }
        } else if c == ';' {
            // https://zh.wikipedia.org/wiki/%E4%B8%8D%E6%8D%A2%E8%A1%8C%E7%A9%BA%E6%A0%BC
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
            if novel_api::is_some_and(last, |last_char| last_char == c) {
                return;
            }
            result.push(c);
        } else {
            result.push(c);
        }
    } else {
        match super::CONVERT_MAP.get(&c) {
            Some(new) => {
                result.push(*new);
            }
            None => result.push(c),
        }
    }
}

fn custom_convert<T>(str: T) -> String
where
    T: AsRef<str>,
{
    let str = html_escape::decode_html_entities(str.as_ref()).to_string();

    let mut result = String::new();
    for (c, next_c) in str.chars().zip(str.chars().skip(1)) {
        do_custom_convert(c, Some(next_c), &mut result);
    }
    do_custom_convert(str.chars().last().unwrap(), None, &mut result);

    result.trim().to_string()
}

pub fn convert_str<T>(str: T, converts: &Vec<Convert>) -> Result<String>
where
    T: AsRef<str>,
{
    if converts.is_empty() {
        return Ok(str.as_ref().to_string());
    } else {
        let mut result = String::new();

        static OPENCC_S2T: OnceCell<OpenCC> = OnceCell::new();
        static OPENCC_T2S: OnceCell<OpenCC> = OnceCell::new();
        static OPENCC_JP2T2S: OnceCell<OpenCC> = OnceCell::new();

        if converts.contains(&Convert::JP2T2S) {
            result = OPENCC_JP2T2S
                .get_or_try_init(|| OpenCC::new(vec![Config::JP2T, Config::T2S]))?
                .convert(&str)?;
        } else if converts.contains(&Convert::T2S) {
            result = OPENCC_T2S
                .get_or_try_init(|| OpenCC::new(vec![Config::T2S]))?
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
        }

        Ok(result)
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
        assert_eq!(convert_str("妳", &config)?, "妳");
        assert_eq!(convert_str("Ｑ０", &config)?, "Q0");
        assert_eq!(convert_str("“安装后”", &config)?, "“安装后”");
        assert_eq!(convert_str("&amp;", &config)?, "&");
        assert_eq!(convert_str("安裝後?", &config)?, "安装后？");
        assert_eq!(convert_str("，，，", &config)?, "，");
        assert_eq!(convert_str("安　装", &config)?, "安 装");
        assert_eq!(convert_str("你\n好", &config)?, "你\n好");

        Ok(())
    }
}
