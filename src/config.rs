use clap::{crate_authors, crate_name, crate_version, value_parser, ArgAction, Parser, Subcommand};
use fluent_templates::Loader;

use crate::{
    cmd::{
        build::Build, check::Check, completions::Completions, download::Download,
        favorites::Favorites, info::Info, real_cugan::RealCugan, search::Search,
        transform::Transform, unzip::Unzip, update::Update, zip::Zip,
    },
    LANG_ID, LOCALES,
};

#[derive(Debug, Parser)]
#[command(author, version, about = about_msg(), long_about = None, propagate_version = true)]
pub struct Config {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, short, action = ArgAction::Count, global = true, default_value_t = 0,
        value_parser = value_parser!(u8).range(0..=5),
        help = LOCALES.lookup(&LANG_ID, "verbose").expect("`verbose` does not exists"))]
    pub verbose: u8,

    #[arg(long, short, global = true, conflicts_with = "verbose", default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "quiet").expect("`quiet` does not exists"))]
    pub quiet: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Download(Download),
    Search(Search),
    Info(Info),
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

fn about_msg() -> &'static str {
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

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_cli() {
        Config::command().debug_assert()
    }
}
