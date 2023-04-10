use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Ok, Result};
use clap::{value_parser, Args};
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, Client, SfacgClient, Timing};
use tokio::sync::Semaphore;
use tracing::{info, warn};
use url::Url;

use crate::cmd::{Convert, Source};
use crate::{utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Args)]
#[command(about = LOCALES.lookup(&LANG_ID, "favorites_command").unwrap())]
pub struct Favorites {
    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source").unwrap())]
    pub source: Source,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").unwrap())]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring").unwrap())]
    pub ignore_keyring: bool,

    #[arg(short, long, default_value_t = 8, value_parser = value_parser!(u8).range(1..=16),
        help = LOCALES.lookup(&LANG_ID, "maximum_concurrency").unwrap())]
    pub maximum_concurrency: u8,

    #[arg(long, num_args = 0..=1, default_missing_value = super::PROXY,
        help = LOCALES.lookup(&LANG_ID, "proxy").unwrap())]
    pub proxy: Option<Url>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "no_proxy").unwrap())]
    pub no_proxy: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = super::default_cert_path(),
        help = super::cert_help_msg())]
    pub cert: Option<PathBuf>,
}

pub async fn execute(config: Favorites) -> Result<()> {
    let mut timing = Timing::new();

    match config.source {
        Source::Sfacg => {
            let mut client = SfacgClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            do_execute(client, config).await?
        }
        Source::Ciweimao => {
            let mut client = CiweimaoClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            do_execute(client, config).await?
        }
    }

    info!("Time spent on `favorites`: {}", timing.elapsed()?);

    Ok(())
}

async fn do_execute<T>(client: T, config: Favorites) -> Result<()>
where
    T: Client + Send + Sync + 'static,
{
    utils::login(&client, &config.source, config.ignore_keyring).await?;
    let novel_ids = client.favorite_infos().await?;

    let mut novel_infos = Vec::new();

    let client = Arc::new(client);
    super::handle_ctrl_c(&client);

    let semaphore = Arc::new(Semaphore::new(config.maximum_concurrency as usize));
    let mut handles = Vec::new();

    for novel_id in novel_ids {
        let client = Arc::clone(&client);
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        handles.push(tokio::spawn(async move {
            let novel_info = client.novel_info(novel_id).await?;
            drop(permit);
            Ok(novel_info)
        }));
    }

    for handle in handles {
        let novel_info = handle.await??;

        if novel_info.is_none() {
            warn!("The novel does not exist, and it may have been taken down");
        } else {
            novel_infos.push(novel_info.unwrap());
        }
    }

    utils::print_novel_infos(novel_infos, &config.converts)?;

    Ok(())
}
