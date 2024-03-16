use std::io;

use clap::{Args, CommandFactory};
use clap_complete::Shell;
use color_eyre::eyre::Result;
use fluent_templates::Loader;
use novel_api::Timing;
use tracing::debug;

use crate::{config::Config, LANG_ID, LOCALES};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "completions_command"))]
pub struct Completions {
    #[arg(value_enum,
        help = LOCALES.lookup(&LANG_ID, "shell"))]
    pub shell: Shell,
}

pub fn execute(config: Completions) -> Result<()> {
    let mut timing = Timing::new();

    let mut cmd = Config::command();
    let bin_name = cmd.get_name().to_string();

    clap_complete::generate(config.shell, &mut cmd, bin_name, &mut io::stdout());

    debug!("Time spent on `completions`: {}", timing.elapsed()?);

    Ok(())
}
