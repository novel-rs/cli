use std::{env, fs};

use assert_cmd::Command;
use color_eyre::eyre::Result;
use ntest::test_case;

mod utils;

#[test]
fn build_pandoc() -> Result<()> {
    do_build_pandoc(false, false)?;
    do_build_pandoc(false, true)?;
    do_build_pandoc(true, false)?;
    do_build_pandoc(true, true)?;
    Ok(())
}

fn do_build_pandoc(delete: bool, in_directory: bool) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let input_path;
    let epub_path;
    if in_directory {
        input_path = utils::copy_to_temp_dir("pandoc", temp_dir.path())?;
        epub_path = env::current_dir()?.join(novel_cli::utils::read_markdown_to_epub_file_name(
            input_path.join("pandoc.md"),
        )?);
    } else {
        input_path = utils::copy_to_temp_dir("pandoc", temp_dir.path())?.join("pandoc.md");
        epub_path = env::current_dir()?.join(novel_cli::utils::read_markdown_to_epub_file_name(
            &input_path,
        )?);
    }

    let mut cmd = Command::cargo_bin("novel-cli")?;
    if delete {
        cmd.args([
            "build",
            "--delete",
            input_path.display().to_string().as_str(),
        ]);
        cmd.assert().success();

        assert!(!input_path.try_exists()?);
    } else {
        cmd.args(["build", input_path.display().to_string().as_str()]);
        cmd.assert().success();

        assert!(input_path.try_exists()?);
    }
    assert!(epub_path.is_file());

    fs::remove_file(epub_path)?;

    Ok(())
}

#[test_case(true)]
#[test_case(false)]
fn do_build_mdbook(delete: bool) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let input_path = utils::copy_to_temp_dir("mdbook", temp_dir.path())?;
    novel_cli::utils::ensure_mdbook_dir(&input_path)?;

    let mut cmd = Command::cargo_bin("novel-cli")?;
    if delete {
        cmd.args([
            "build",
            "--delete",
            input_path.display().to_string().as_str(),
        ]);
        cmd.assert().success();

        assert!(input_path.join("index.html").is_file());
        assert!(!input_path.join("book").try_exists()?);
        assert!(!input_path.join("src").try_exists()?);
        assert!(!input_path.join("book.toml").try_exists()?);
    } else {
        cmd.args(["build", input_path.display().to_string().as_str()]);
        cmd.assert().success();

        assert!(input_path.join("book").join("index.html").is_file());
        novel_cli::utils::ensure_mdbook_dir(&input_path)?;
    }

    Ok(())
}
