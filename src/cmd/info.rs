use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, Client, SfacgClient, Timing};
use tracing::{info, warn};
use url::Url;

use crate::cmd::{Convert, Source};
use crate::{utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "info_command").unwrap())]
pub struct Info {
    #[arg(help = LOCALES.lookup(&LANG_ID, "novel_id").unwrap())]
    pub novel_id: u32,

    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source").unwrap())]
    pub source: Source,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").unwrap())]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring").unwrap())]
    pub ignore_keyring: bool,

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

pub async fn execute(config: Info) -> Result<()> {
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
            utils::login(&client, &config.source, config.ignore_keyring).await?;
            do_execute(client, config).await?
        }
    }

    info!("Time spent on `info`: {}", timing.elapsed()?);

    Ok(())
}

async fn do_execute<T>(client: T, config: Info) -> Result<()>
where
    T: Client + Send + Sync + 'static,
{
    let client = Arc::new(client);
    super::handle_ctrl_c(&client);

    let novel_info = utils::novel_info(&client, config.novel_id).await?;

    if let Some(ref url) = novel_info.cover_url {
        match client.image(url).await {
            Ok(image) => utils::print_novel_info(Some(image), novel_info, &config.converts)?,
            Err(error) => warn!("{error}"),
        }
    } else {
        utils::print_novel_info(None, novel_info, &config.converts)?;
    }

    Ok(())
}
