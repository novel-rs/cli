use std::io::{self, Write};

use assert_cmd::Command;
use color_eyre::eyre::Result;

mod utils;

#[test]
fn completions() -> Result<()> {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd.args(["completions", "zsh"]).output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}
