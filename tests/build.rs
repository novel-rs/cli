use std::{env, fs};

use assert_cmd::Command;
use rstest::rstest;
use serial_test::file_serial;
use testresult::TestResult;

mod utils;

#[rstest]
#[case(false, false)]
#[case(true, false)]
#[case(false, true)]
#[case(true, true)]
#[file_serial(build_pandoc)]
fn build_pandoc(#[case] delete: bool, #[case] in_directory: bool) -> TestResult {
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

#[rstest]
#[case(false)]
#[case(true)]
fn build_mdbook(#[case] delete: bool) -> TestResult {
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
