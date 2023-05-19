use std::{env, fs};

use assert_cmd::Command;
use color_eyre::eyre::Result;

mod utils;

#[test]
fn zip() -> Result<()> {
    do_zip(false)?;
    do_zip(true)?;

    Ok(())
}

fn do_zip(delete: bool) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let input_path = utils::copy_to_temp_dir("pandoc-epub", temp_dir.path())?;
    novel_cli::utils::ensure_epub_dir(&input_path)?;

    let mut cmd = Command::cargo_bin("novel-cli")?;
    if delete {
        cmd.args(["zip", "--delete", input_path.display().to_string().as_str()]);
        cmd.assert().success();

        assert!(!input_path.try_exists()?);
    } else {
        cmd.args(["zip", input_path.display().to_string().as_str()]);
        cmd.assert().success();

        assert!(input_path.is_dir());
    }
    let epub_path = env::current_dir()?.join("pandoc-epub.epub");
    assert!(epub_path.is_file());

    fs::remove_file(epub_path)?;

    Ok(())
}
