use std::path::{Path, PathBuf};

use color_eyre::eyre::{bail, ensure, Result};

pub fn ensure_executable_exists<T>(name: T) -> Result<()>
where
    T: AsRef<str>,
{
    let name = name.as_ref();

    if let Err(error) = which::which(name) {
        bail!("{}: `{}`", error, name);
    }

    Ok(())
}

pub fn ensure_markdown_file<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    if !is_markdown_file(&path)? {
        bail!("File `{}` is not markdown file", path.as_ref().display())
    }

    Ok(())
}

pub fn ensure_epub_file<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    if !is_epub_file(&path)? {
        bail!("File `{}` is not epub file", path.as_ref().display())
    }

    Ok(())
}

pub fn ensure_epub_dir<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    if !is_epub_dir(&path)? {
        bail!(
            "Directory `{}` is not epub directory",
            path.as_ref().display()
        )
    }

    Ok(())
}

pub fn ensure_pandoc_dir<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    if !is_pandoc_dir(&path)? {
        bail!(
            "Directory `{}` is not pandoc directory",
            path.as_ref().display()
        )
    }

    Ok(())
}

pub fn ensure_mdbook_dir<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    if !is_mdbook_dir(&path)? {
        bail!(
            "Directory `{}` is not mdbook directory",
            path.as_ref().display()
        )
    }

    Ok(())
}

pub fn is_markdown_file<T>(path: T) -> Result<bool>
where
    T: AsRef<Path>,
{
    is_some_file(path, "md")
}

pub fn is_epub_file<T>(path: T) -> Result<bool>
where
    T: AsRef<Path>,
{
    is_some_file(path, "epub")
}

pub fn is_markdown_dir<T>(path: T) -> Result<bool>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    ensure_exists(path)?;

    if !path.is_dir() {
        return Ok(false);
    }

    let markdown_file_name = PathBuf::from(path.file_stem().unwrap()).with_extension("md");
    let markdown_file_path = path.join(markdown_file_name);

    Ok(markdown_file_path.try_exists()?)
}

pub fn is_mdbook_dir<T>(path: T) -> Result<bool>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    ensure_exists(path)?;

    if !path.is_dir() {
        return Ok(false);
    }

    let src_path = path.join("src");
    let toml_path = path.join("book.toml");

    Ok(src_path.is_dir() && toml_path.is_file())
}

pub fn is_pandoc_dir<T>(path: T) -> Result<bool>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    ensure_exists(path)?;

    if !path.is_dir() {
        return Ok(false);
    }

    let markdown = path.join(path.file_stem().unwrap()).with_extension("md");

    Ok(markdown.is_file())
}

pub fn is_epub_dir<T>(path: T) -> Result<bool>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    ensure_exists(path)?;

    if !path.is_dir() {
        return Ok(false);
    }

    let epub_path = path.join("EPUB");
    let meta_path = path.join("META-INF");
    let mimetype_path = path.join("mimetype");

    Ok(epub_path.is_dir() && meta_path.is_dir() && mimetype_path.is_file())
}

fn is_some_file<T, E>(path: T, extension: E) -> Result<bool>
where
    T: AsRef<Path>,
    E: AsRef<str>,
{
    let path = path.as_ref();

    ensure_exists(path)?;

    if !path.is_file() {
        return Ok(false);
    }

    Ok(path
        .extension()
        .is_some_and(|ext| ext == extension.as_ref()))
}

fn ensure_exists<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    ensure!(
        path.try_exists()?,
        "File or directory `{}` does not exist",
        path.display()
    );

    Ok(())
}
