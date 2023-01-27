mod mdbook;
mod pandoc;

pub(crate) use self::mdbook::*;
pub(crate) use self::pandoc::*;

use std::path::Path;

#[must_use]
fn image_markdown_str<T>(path: T) -> String
where
    T: AsRef<Path>,
{
    format!("![]({})", path.as_ref().display())
}
