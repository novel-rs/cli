mod mdbook;
mod pandoc;

pub use self::mdbook::*;
pub use self::pandoc::*;

use std::path::Path;

use crate::cmd::Convert;

pub fn image_markdown_str<T>(path: T) -> String
where
    T: AsRef<Path>,
{
    format!("![]({})", path.as_ref().display())
}

fn lang(convert: &[Convert]) -> String {
    if convert.contains(&Convert::S2T) {
        String::from("zh-Hant")
    } else {
        String::from("zh-Hans")
    }
}
