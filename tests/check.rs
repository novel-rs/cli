use std::io::{self, Write};

use anyhow::Result;
use assert_cmd::Command;
use fs_extra::dir::CopyOptions;

mod utils;

#[test]
fn check_pandoc() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let test_data_path = utils::test_data_path()?.join("pandoc");

    let mut options = CopyOptions::new();
    options.copy_inside = true;
    fs_extra::dir::copy(test_data_path, temp_dir.path(), &options)?;

    let input_path = temp_dir.path().join("pandoc").join("pandoc.md");
    assert!(input_path.is_file());

    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd
        .args(["check", input_path.display().to_string().as_str()])
        .output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}
