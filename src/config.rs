use std::{env, path::PathBuf};

use clap::{crate_authors, crate_name, crate_version, value_parser, ArgAction, Parser, Subcommand};
use fluent_templates::Loader;

use crate::{
    cmd::{
        build::Build, check::Check, completions::Completions, download::Download,
        favorites::Favorites, info::Info, read::Read, real_cugan::RealCugan, search::Search,
        transform::Transform, unzip::Unzip, update::Update, zip::Zip,
    },
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Parser)]
#[command(author, version = version_msg(), about = about_msg(), long_about = None, propagate_version = true)]
pub struct Config {
    #[command(subcommand)]
    pub command: Commands,

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
    Favorites(Favorites),
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
    format!(
        "{}\nExecutable Path: {}\nConfig Dir: {}\nData Dir: {}",
        crate_version!(),
        env::current_exe()
            .unwrap_or_else(|_| {
                eprintln!("Unable to get current executable path");
                PathBuf::from(crate_name!())
            })
            .display(),
        novel_api::config_dir_path("").unwrap().display(),
        novel_api::data_dir_path("").unwrap().display()
    )
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
