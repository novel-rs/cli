use anyhow::{bail, Result};

const WINDOWS_LINE_BREAK: &str = "\r\n";
const UNIX_LINE_BREAK: &str = "\n";

#[cfg(not(target_os = "windows"))]
pub const LINE_BREAK: &str = UNIX_LINE_BREAK;
#[cfg(target_os = "windows")]
pub const LINE_BREAK: &str = WINDOWS_LINE_BREAK;

pub fn verify_line_break<T>(text: T) -> Result<()>
where
    T: AsRef<str>,
{
    let text = text.as_ref();

    if cfg!(target_os = "windows")
        && text.contains(UNIX_LINE_BREAK)
        && !text.contains(WINDOWS_LINE_BREAK)
    {
        bail!(r"The line break under Windows should be `\r\n`");
    } else if cfg!(not(target_os = "windows")) && text.contains(WINDOWS_LINE_BREAK) {
        bail!(r"The line break under Unix should be `\n`");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_break() -> Result<()> {
        #[cfg(not(target_os = "windows"))]
        verify_line_break("12345\n\n123")?;

        #[cfg(target_os = "windows")]
        verify_line_break("12345\r\n\r\n")?;

        Ok(())
    }

    #[test]
    #[should_panic]
    fn line_break_failed() {
        #[cfg(not(target_os = "windows"))]
        verify_line_break("12345\r\n\n123").unwrap();

        #[cfg(target_os = "windows")]
        verify_line_break("12345\n\r\n").unwrap();
    }
}
