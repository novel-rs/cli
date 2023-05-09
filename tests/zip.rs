use anyhow::Result;
use assert_cmd::Command;
use fs_extra::dir::CopyOptions;

mod utils;

#[test]
fn zip() -> Result<()> {
    do_zip(false)
}

#[test]
fn zip_delete() -> Result<()> {
    do_zip(true)
}

fn do_zip(delete: bool) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let test_data_path = utils::test_data_path()?.join("pandoc-epub");

    let mut options = CopyOptions::new();
    options.copy_inside = true;
    fs_extra::dir::copy(test_data_path, temp_dir.path(), &options)?;

    let input_path = temp_dir.path().join("pandoc-epub");
    assert!(input_path.is_dir());

    let mut cmd = Command::cargo_bin("novel-cli")?;
    if delete {
        cmd.args(["zip", "--delete", input_path.display().to_string().as_str()]);
        cmd.assert().success();

        assert!(!input_path.try_exists()?);
    } else {
        cmd.args(["zip", input_path.display().to_string().as_str()]);
        cmd.assert().success();

        assert!(input_path.is_dir());
    }
    let epub_path = temp_dir.path().join("pandoc-epub.epub");
    assert!(epub_path.is_file());

    Ok(())
}
