use std::{
    fs,
    io::{self, Write},
};

use assert_cmd::Command;
use color_eyre::eyre::Result;
use serial_test::file_serial;

const NOVEL_NAME: &str = "转生精灵公主可以备受宠爱吗？";

#[test]
#[file_serial(download)]
fn download_pandoc() -> Result<()> {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd
        .args([
            "download",
            "--source=sfacg",
            "--format=pandoc",
            "--skip-login",
            "--backtrace=full",
            "548678",
        ])
        .output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    novel_cli::utils::ensure_pandoc_dir(NOVEL_NAME)?;
    fs::remove_dir_all(NOVEL_NAME)?;

    Ok(())
}

#[test]
#[file_serial(download)]
fn download_mdbook() -> Result<()> {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd
        .args([
            "download",
            "--source=sfacg",
            "--format=mdbook",
            "--skip-login",
            "--backtrace=full",
            "548678",
        ])
        .output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    novel_cli::utils::ensure_mdbook_dir(NOVEL_NAME)?;
    fs::remove_dir_all(NOVEL_NAME)?;

    Ok(())
}
