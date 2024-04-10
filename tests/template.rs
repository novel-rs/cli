use std::{
    fs,
    io::{self, Write},
};

use assert_cmd::Command;
use testresult::TestResult;

#[test]
fn template() -> TestResult {
    let novel_name = "你好世界";

    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd.args(["template", novel_name]).output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    novel_cli::utils::ensure_pandoc_dir(novel_name)?;
    fs::remove_dir_all(novel_name)?;

    Ok(())
}
