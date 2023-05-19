use std::{env, fs};

use assert_cmd::Command;
use color_eyre::eyre::Result;

mod utils;

#[test]
fn unzip() -> Result<()> {
    do_unzip(false)?;
    do_unzip(true)?;

    Ok(())
}

fn do_unzip(delete: bool) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let input_path = utils::copy_to_temp_dir("pandoc-epub.epub", temp_dir.path())?;

    let mut cmd = Command::cargo_bin("novel-cli")?;
    if delete {
        cmd.args([
            "unzip",
            "--delete",
            input_path.display().to_string().as_str(),
        ]);
        cmd.assert().success();

        assert!(!input_path.try_exists()?);
    } else {
        cmd.args(["unzip", input_path.display().to_string().as_str()]);
        cmd.assert().success();

        assert!(input_path.is_file());
    }
    let epub_dir_path = env::current_dir()?.join("pandoc-epub");
    novel_cli::utils::ensure_epub_dir(&epub_dir_path)?;

    fs::remove_dir_all(epub_dir_path)?;

    Ok(())
}
