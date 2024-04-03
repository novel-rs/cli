use std::io::{self, Write};

use assert_cmd::Command;
use testresult::TestResult;

#[test]
fn version() -> TestResult {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd.arg("-V").output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}
