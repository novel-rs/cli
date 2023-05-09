use std::fs;

use anyhow::Result;
use assert_cmd::Command;

mod utils;

#[test]
fn unzip() -> Result<()> {
    do_unzip(false)
}

#[test]
fn unzip_delete() -> Result<()> {
    do_unzip(true)
}

fn do_unzip(delete: bool) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let test_data_path = utils::test_data_path()?.join("pandoc-epub.epub");
    let input_path = temp_dir.path().join("pandoc-epub.epub");

    fs::copy(test_data_path, &input_path)?;
    assert!(input_path.is_file());

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
    let epub_dir_path = temp_dir.path().join("pandoc-epub");
    assert!(epub_dir_path.is_dir());

    Ok(())
}
