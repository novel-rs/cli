use std::{
    env,
    io::{self, IsTerminal},
};

use bytesize::ByteSize;
use clap::Parser;
use color_eyre::eyre::Result;
use memory_stats::memory_stats;
use mimalloc::MiMalloc;
use novel_cli::{
    cmd::{
        self, bookshelf, build, check, completions, download, info, read, real_cugan, search, sign,
        template, transform, unzip, update,
    },
    config::{Backtrace, Commands, Config},
};
use supports_color::Stream;
use tracing::{debug, error, info, subscriber};
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_log::LogTracer;
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();

    let _guard = init_log(&config)?;

    debug!("{:#?}", sys_locale::get_locale());

    novel_cli::init_error_hooks(matches!(&config.command, Commands::Read(_)))?;

    if !matches!(&config.command, Commands::Completions(_)) && !io::stdout().is_terminal() {
        error!("stdout must be a terminal");
    }

    match config.command {
        Commands::Sign(config) => sign::execute(config).await?,
        Commands::Download(config) => download::execute(config).await?,
        Commands::Search(config) => search::execute(config).await?,
        Commands::Info(config) => info::execute(config).await?,
        Commands::Read(config) => read::execute(config).await?,
        Commands::Bookshelf(config) => bookshelf::execute(config).await?,
        Commands::Template(config) => template::execute(config)?,
        Commands::Transform(config) => transform::execute(config)?,
        Commands::Check(config) => check::execute(config)?,
        Commands::Build(config) => build::execute(config)?,
        Commands::Zip(config) => cmd::zip::execute(config)?,
        Commands::Unzip(config) => unzip::execute(config)?,
        Commands::RealCugan(config) => real_cugan::execute(config).await?,
        Commands::Update(config) => update::execute(config).await?,
        Commands::Completions(config) => completions::execute(config)?,
    }

    if config.verbose >= 1 {
        if let Some(usage) = memory_stats() {
            info!(
                "Current physical memory usage: {}",
                ByteSize(usage.physical_mem as u64)
            );
        } else {
            error!("Couldn't get the current memory usage");
        }
    }

    Ok(())
}

fn init_log(config: &Config) -> Result<Option<WorkerGuard>> {
    if config.backtrace.is_some() {
        match config.backtrace.as_ref().unwrap() {
            Backtrace::ON => env::set_var("RUST_BACKTRACE", "1"),
            Backtrace::FULL => env::set_var("RUST_BACKTRACE", "full"),
        }
    }

    if config.verbose == 4 {
        LogTracer::init()?;
    }

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

    let mut guard = None;
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(EnvFilter::from_default_env());

    if config.output_log_to_file {
        let file_appender = rolling::never(".", "novel-cli.log");
        let (non_blocking, worker_guard) = tracing_appender::non_blocking(file_appender);

        guard = Some(worker_guard);

        let subscriber = subscriber
            .with_ansi(false)
            .with_writer(non_blocking)
            .finish();
        subscriber::set_global_default(subscriber)?;
    } else {
        let subscriber = subscriber
            .with_ansi(supports_color::on(Stream::Stdout).is_some())
            .finish();
        subscriber::set_global_default(subscriber)?;
    }

    Ok(guard)
}
