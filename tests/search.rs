use std::io::{self, Write};

use assert_cmd::Command;
use testresult::TestResult;

mod utils;

#[test]
fn search_show_tags() -> TestResult {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd
        .args(["search", "--source=sfacg", "--show-tags"])
        .output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}

#[test]
fn search_show_categories() -> TestResult {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd
        .args(["search", "--source=sfacg", "--show-categories"])
        .output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}

#[test]
fn search() -> TestResult {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd
        .args([
            "search",
            "--source=sfacg",
            "--tags=百合",
            "--is-finished=false",
            "--update-days=7",
            "--category=魔幻",
            "--min-word-count=1000000",
            "--limit=10",
            "--excluded-tags=搞笑,综漫,同人",
        ])
        .output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}
