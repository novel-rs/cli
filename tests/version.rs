use std::io::{self, Write};

use assert_cmd::Command;
use color_eyre::eyre::Result;

#[test]
fn version() -> Result<()> {
    let mut cmd = Command::cargo_bin("novel-cli")?;
    let output = cmd.arg("-V").output()?;
    cmd.assert().success();

    io::stderr().write_all(&output.stdout)?;

    Ok(())
}
