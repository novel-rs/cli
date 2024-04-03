use std::io::{self, Write};

use assert_cmd::Command;
use testresult::TestResult;

#[test]
fn help() -> TestResult {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd.arg("-h").output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}

#[test]
fn download_help() -> TestResult {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd.args(["download", "-h"]).output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}
