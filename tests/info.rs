use std::io::{self, Write};

use assert_cmd::Command;
use testresult::TestResult;

#[test]
fn info() -> TestResult {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd.args(["info", "--source=sfacg", "548678"]).output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}
