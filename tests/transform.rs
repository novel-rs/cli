use std::fs;

use anyhow::Result;
use assert_cmd::Command;
use fs_extra::dir::CopyOptions;

mod utils;

#[test]
fn transform() -> Result<()> {
    do_transform(false)
}

#[test]
fn transform_delete() -> Result<()> {
    do_transform(true)
}

fn do_transform(delete: bool) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let test_data_path = utils::test_data_path()?.join("pandoc");

    let mut options = CopyOptions::new();
    options.copy_inside = true;
    fs_extra::dir::copy(test_data_path, temp_dir.path(), &options)?;

    let input_path = temp_dir.path().join("pandoc").join("pandoc.md");
    assert!(input_path.is_file());

    let output_path_old = temp_dir.path().join("pandoc").join("pandoc.old.md");
    let (meta_data, _) = novel_cli::utils::read_markdown(&input_path)?;

    let mut cmd = Command::cargo_bin("novel-cli")?;
    if delete {
        cmd.args([
            "transform",
            "--converts=custom",
            "--delete",
            input_path.display().to_string().as_str(),
        ]);
        cmd.assert().success();

        assert!(!output_path_old.try_exists()?);
    } else {
        cmd.args([
            "transform",
            "--converts=custom",
            input_path.display().to_string().as_str(),
        ]);
        cmd.assert().success();

        assert!(output_path_old.is_file());
    }

    let novel_name =
        novel_cli::utils::convert_str(meta_data.title, [novel_cli::cmd::Convert::CUSTOM])?;
    let output_file_name = temp_dir
        .path()
        .join("pandoc")
        .join(novel_cli::utils::to_markdown_file_name(novel_name));
    assert!(output_file_name.is_file());

    let content = fs::read_to_string(output_file_name)?;
    novel_cli::utils::verify_line_break(content)?;

    Ok(())
}
