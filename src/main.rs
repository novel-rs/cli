use std::env;

use anyhow::Result;
use bytesize::ByteSize;
use clap::Parser;
use memory_stats::memory_stats;
use snmalloc_rs::SnMalloc;
use tracing::{debug, info, subscriber, warn};
use tracing_log::LogTracer;
use tracing_subscriber::EnvFilter;

use novel_cli::{
    cmd::{
        self, build, check, completions, download, favorites, info, real_cugan, search, transform,
        unzip, update,
    },
    config::{Commands, Config},
    LANG_ID,
};

#[global_allocator]
static ALLOC: SnMalloc = SnMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();

    init_log(&config)?;

    debug!("{config:#?}");

    debug!("{:#?}", sys_locale::get_locale());
    debug!("{LANG_ID:#?}");

    match config.command {
        Commands::Download(config) => download::execute(config).await?,
        Commands::Search(config) => search::execute(config).await?,
        Commands::Info(config) => info::execute(config).await?,
        Commands::Favorites(config) => favorites::execute(config).await?,
        Commands::Transform(config) => transform::execute(config)?,
        Commands::Check(config) => check::execute(config)?,
        Commands::Build(config) => build::execute(config)?,
        Commands::Zip(config) => cmd::zip::execute(config)?,
        Commands::Unzip(config) => unzip::execute(config)?,
        Commands::RealCugan(config) => real_cugan::execute(config).await?,
        Commands::Update(config) => update::execute(config)?,
        Commands::Completions(config) => completions::execute(config)?,
    }

    if config.verbose >= 1 {
        if let Some(usage) = memory_stats() {
            info!(
                "Current physical memory usage: {}",
                ByteSize(usage.physical_mem as u64)
            );
        } else {
            warn!("Couldn't get the current memory usage");
        }
    }

    Ok(())
}

fn init_log(config: &Config) -> Result<()> {
    LogTracer::init()?;

    let rust_log = if config.quiet {
        "none"
    } else if config.verbose == 1 {
        "none,novel_api=info,novel_cli=info"
    } else if config.verbose == 2 {
        "none,novel_api=debug,novel_cli=debug"
    } else if config.verbose == 3 {
        "none,novel_api=trace,novel_cli=trace"
    } else if config.verbose == 4 {
        "trace"
    } else {
        "none,novel_api=warn,novel_cli=warn"
    };

    env::set_var("RUST_LOG", rust_log);

    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    subscriber::set_global_default(subscriber)?;

    Ok(())
}
