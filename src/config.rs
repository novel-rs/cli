use std::{env, path::PathBuf};

use anstyle::{AnsiColor, Color};
use clap::{
    builder::Styles, crate_authors, crate_name, crate_version, value_parser, ArgAction, Parser,
    Subcommand, ValueEnum,
};
use fluent_templates::Loader;
use shadow_rs::shadow;
use supports_color::Stream;

use crate::{
    cmd::{
        bookshelf::Bookshelf, build::Build, check::Check, completions::Completions,
        download::Download, info::Info, read::Read, real_cugan::RealCugan, search::Search,
        transform::Transform, unzip::Unzip, update::Update, zip::Zip,
    },
    LANG_ID, LOCALES,
};

shadow!(shadow_build);

#[must_use]
#[derive(Parser)]
#[command(author, version = version_msg(), about = about_msg(),
    long_about = None, propagate_version = true, styles = get_styles())]
pub struct Config {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, value_enum, global = true,
        help = LOCALES.lookup(&LANG_ID, "backtrace").unwrap())]
    pub backtrace: Option<Backtrace>,

    #[arg(short, long, action = ArgAction::Count, global = true, default_value_t = 0,
        value_parser = value_parser!(u8).range(0..=4),
        help = LOCALES.lookup(&LANG_ID, "verbose").unwrap())]
    pub verbose: u8,

    #[arg(short, long, global = true, conflicts_with = "verbose", default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "quiet").unwrap())]
    pub quiet: bool,
}

#[must_use]
#[derive(Subcommand)]
pub enum Commands {
    Download(Download),
    Search(Search),
    Info(Info),
    Read(Read),
    Bookshelf(Bookshelf),
    Transform(Transform),
    Check(Check),
    Build(Build),
    Zip(Zip),
    Unzip(Unzip),
    RealCugan(RealCugan),
    Update(Update),
    Completions(Completions),
}

#[must_use]
#[derive(Clone, PartialEq, ValueEnum)]
pub enum Backtrace {
    ON,
    FULL,
}

#[must_use]
const fn about_msg() -> &'static str {
    concat!(
        crate_name!(),
        " ",
        crate_version!(),
        "\nAuthor: ",
        crate_authors!(),
        "\nProject home page: ",
        env!("CARGO_PKG_HOMEPAGE"),
    )
}

#[must_use]
fn version_msg() -> String {
    let version = crate_version!();
    let author = crate_authors!();
    let home_page = env!("CARGO_PKG_HOMEPAGE");

    let commit_date = shadow_build::COMMIT_DATE;
    let commit_hash = shadow_build::COMMIT_HASH;
    let build_time = shadow_build::BUILD_TIME;
    let build_target = shadow_build::BUILD_TARGET;

    let os_info = os_info::get();
    let architecture = os_info.architecture().unwrap_or("unknown");

    let current_exe_path = env::current_exe()
        .unwrap_or_else(|_| {
            eprintln!("Unable to get current executable path");
            PathBuf::from(crate_name!())
        })
        .display()
        .to_string();
    let config_dir_path = novel_api::config_dir_path("some-source")
        .unwrap()
        .display()
        .to_string();
    let data_dir_path = novel_api::data_dir_path("some-source")
        .unwrap()
        .display()
        .to_string();

    format!(
        "\
{version}
Author: {author}
Project home page: {home_page}

Commit date: {commit_date}
Commit hash: {commit_hash}
Build time: {build_time}
Build target: {build_target}

OS information: {os_info} [{architecture}]

Executable path: {current_exe_path}
Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}

pub fn get_styles() -> Styles {
    if supports_color::on(Stream::Stdout).is_some() {
        Styles::styled()
            .header(
                anstyle::Style::new()
                    .bold()
                    .underline()
                    .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
            )
            .usage(
                anstyle::Style::new()
                    .bold()
                    .underline()
                    .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
            )
            .literal(
                anstyle::Style::new()
                    .bold()
                    .fg_color(Some(Color::Ansi(AnsiColor::Green))),
            )
            .placeholder(anstyle::Style::new().fg_color(Some(Color::Ansi(AnsiColor::Blue))))
    } else {
        Styles::plain()
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_cli() {
        Config::command().debug_assert()
    }
}
