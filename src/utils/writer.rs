use std::{io::Cursor, path::Path};

use anyhow::Result;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

#[must_use]
pub(crate) struct Writer {
    writer: BufWriter<File>,
}

impl Writer {
    pub(crate) async fn new<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        Ok(Self {
            writer: BufWriter::new(File::create(&path).await?),
        })
    }

    #[inline]
    pub(crate) async fn write<T>(&mut self, text: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        let mut buffer = Cursor::new(text.as_ref());
        self.writer.write_all_buf(&mut buffer).await?;
        Ok(())
    }

    #[inline]
    pub(crate) async fn ln(&mut self) -> Result<()> {
        self.writer.write_all(b"\n").await?;
        Ok(())
    }

    #[inline]
    pub(crate) async fn writeln<T>(&mut self, text: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        self.write(text).await?;
        self.ln().await?;
        Ok(())
    }

    #[inline]
    pub(crate) async fn flush(&mut self) -> Result<()> {
        self.writer.flush().await?;
        Ok(())
    }
}
