mod mdbook;
mod pandoc;

pub use self::mdbook::*;
pub use self::pandoc::*;

#[inline]
#[must_use]
fn image_markdown_str<T>(path: T) -> String
where
    T: AsRef<str>,
{
    format!("![]({})", path.as_ref())
}
