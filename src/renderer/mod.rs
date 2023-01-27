mod mdbook;
mod pandoc;

pub(crate) use self::mdbook::*;
pub(crate) use self::pandoc::*;

use std::path::Path;

use crate::cmd::Convert;

#[must_use]
fn image_markdown_str<T>(path: T) -> String
where
    T: AsRef<Path>,
{
    format!("![]({})", path.as_ref().display())
}

#[must_use]
fn lang(convert: &[Convert]) -> String {
    if convert.contains(&Convert::S2T) {
        String::from("zh-Hant")
    } else {
        String::from("zh-Hans")
    }
}
