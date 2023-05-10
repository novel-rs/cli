use std::io::{self, Write};

use anyhow::Result;
use assert_cmd::Command;

mod utils;

#[test]
fn check() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let input_path = utils::copy_to_temp_dir("pandoc", temp_dir.path())?.join("pandoc.md");

    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd
        .args(["check", input_path.display().to_string().as_str()])
        .output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}
