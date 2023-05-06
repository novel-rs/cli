use anyhow::Result;
use assert_cmd::Command;
use fs_extra::dir::CopyOptions;

mod utils;

#[test]
fn build_pandoc() -> Result<()> {
    do_build_pandoc(false)
}

#[test]
fn build_pandoc_delete() -> Result<()> {
    do_build_pandoc(true)
}

#[test]
fn build_mdbook() -> Result<()> {
    do_build_mdbook(false)
}

#[test]
fn build_mdbook_delete() -> Result<()> {
    do_build_mdbook(true)
}

fn do_build_pandoc(delete: bool) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let test_data_path = utils::test_data_path()?.join("pandoc");

    let mut options = CopyOptions::new();
    options.copy_inside = true;
    fs_extra::dir::copy(test_data_path, temp_dir.path(), &options)?;

    let input_path = temp_dir.path().join("pandoc");
    assert!(input_path.is_dir());

    let mut cmd = Command::cargo_bin("novel-cli")?;
    if delete {
        cmd.args([
            "build",
            "--format=pandoc",
            "--delete",
            input_path.display().to_string().as_str(),
        ]);
        cmd.assert().success();

        assert!(!input_path.try_exists()?);
    } else {
        cmd.args([
            "build",
            "--format=pandoc",
            input_path.display().to_string().as_str(),
        ]);
        cmd.assert().success();

        assert!(input_path.is_dir());
    }
    let epub_path = temp_dir.path().join("pandoc.epub");
    assert!(epub_path.is_file());

    Ok(())
}

fn do_build_mdbook(delete: bool) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let test_data_path = utils::test_data_path()?.join("mdbook");

    let mut options = CopyOptions::new();
    options.copy_inside = true;
    fs_extra::dir::copy(test_data_path, temp_dir.path(), &options)?;

    let input_path = temp_dir.path().join("mdbook");
    assert!(input_path.is_dir());
    assert!(input_path.join("src").is_dir());
    assert!(input_path.join("book.toml").is_file());

    let mut cmd = Command::cargo_bin("novel-cli")?;
    if delete {
        cmd.args([
            "build",
            "--format=mdbook",
            "--delete",
            input_path.display().to_string().as_str(),
        ]);
        cmd.assert().success();

        assert!(input_path.join("index.html").is_file());
        assert!(!input_path.join("book").try_exists()?);
        assert!(!input_path.join("src").try_exists()?);
        assert!(!input_path.join("book.toml").try_exists()?);
    } else {
        cmd.args([
            "build",
            "--format=mdbook",
            input_path.display().to_string().as_str(),
        ]);
        cmd.assert().success();

        assert!(input_path.join("book").join("index.html").is_file());
        assert!(input_path.join("book").is_dir());
        assert!(input_path.join("src").is_dir());
        assert!(input_path.join("book.toml").is_file());
    }

    Ok(())
}
