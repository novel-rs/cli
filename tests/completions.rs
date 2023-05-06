use std::io::{self, Write};

use anyhow::Result;
use assert_cmd::Command;

mod utils;

#[test]
fn check_pandoc() -> Result<()> {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd.args(["completions", "zsh"]).output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}
