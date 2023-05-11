use anyhow::{bail, Result};
use memchr::memmem;
use rayon::prelude::*;

pub const WINDOWS_LINE_BREAK: &str = "\r\n";
pub const UNIX_LINE_BREAK: &str = "\n";

#[cfg(not(target_os = "windows"))]
pub const LINE_BREAK: &str = UNIX_LINE_BREAK;
#[cfg(target_os = "windows")]
pub const LINE_BREAK: &str = WINDOWS_LINE_BREAK;

pub fn verify_line_break<T>(text: T) -> Result<()>
where
    T: AsRef<str>,
{
    if cfg!(target_os = "windows") {
        let text = text.as_ref();

        text.chars()
            .zip(text.chars().skip(1))
            .par_bridge()
            .try_for_each(|(first, second)| {
                if second == '\n' && first != '\r' {
                    bail!(r"The line break under Windows should be `\r\n`");
                }

                Ok(())
            })?;
    } else if cfg!(not(target_os = "windows")) {
        let text = text.as_ref().as_bytes();

        if memmem::find(text, WINDOWS_LINE_BREAK.as_bytes()).is_some() {
            bail!(r"The line break under Unix should be `\n`");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::assert_panics;

    #[test]
    fn line_break() -> Result<()> {
        #[cfg(not(target_os = "windows"))]
        verify_line_break("12345\n\n123")?;

        #[cfg(target_os = "windows")]
        verify_line_break("12345\r\n\r\n")?;

        Ok(())
    }

    #[test]
    fn line_break_failed() -> Result<()> {
        #[cfg(not(target_os = "windows"))]
        {
            assert_panics!({
                verify_line_break("12345\r\n\n123").unwrap();
            });
            assert_panics!({
                verify_line_break("12345\r\n").unwrap();
            });
        }

        #[cfg(target_os = "windows")]
        {
            assert_panics!({
                verify_line_break("12345\n\r\n").unwrap();
            });
            assert_panics!({
                verify_line_break("12345\n\n").unwrap();
            });
        }

        Ok(())
    }
}
