use anyhow::Result;
use assert_cmd::Command;

#[test]
fn func() -> Result<()> {
    let mut cmd = Command::cargo_bin("novel-cli")?;

    cmd.args(["help", "download"]);

    cmd.assert().success();

    Ok(())
}
