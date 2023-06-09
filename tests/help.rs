use std::io::{self, Write};

use assert_cmd::Command;
use color_eyre::eyre::Result;

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd.arg("-h").output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}

#[test]
fn download_help() -> Result<()> {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd.args(["download", "-h"]).output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}
