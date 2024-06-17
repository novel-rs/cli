use std::{path::PathBuf, sync::Arc};

use clap::Args;
use color_eyre::eyre::Result;
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, CiyuanjiClient, Client, SfacgClient};
use tracing::error;
use url::Url;

use crate::{
    cmd::{Convert, Source},
    utils, LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "info_command"))]
pub struct Info {
    #[arg(help = LOCALES.lookup(&LANG_ID, "novel_id"))]
    pub novel_id: u32,

    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source"))]
    pub source: Source,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts"))]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring"))]
    pub ignore_keyring: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = super::DEFAULT_PROXY,
        help = LOCALES.lookup(&LANG_ID, "proxy"))]
    pub proxy: Option<Url>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "no_proxy"))]
    pub no_proxy: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = super::default_cert_path(),
        help = super::cert_help_msg())]
    pub cert: Option<PathBuf>,
}

pub async fn execute(config: Info) -> Result<()> {
    match config.source {
        Source::Sfacg => {
            let mut client = SfacgClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            do_execute(client, config).await?
        }
        Source::Ciweimao => {
            let mut client = CiweimaoClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            utils::log_in(&client, &config.source, config.ignore_keyring).await?;
            do_execute(client, config).await?
        }
        Source::Ciyuanji => {
            let mut client = CiyuanjiClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            utils::log_in_without_password(&client).await?;
            do_execute(client, config).await?
        }
    }

    Ok(())
}

async fn do_execute<T>(client: T, config: Info) -> Result<()>
where
    T: Client + Send + Sync + 'static,
{
    let client = Arc::new(client);
    super::handle_ctrl_c(&client);

    let novel_info = utils::novel_info(&client, config.novel_id).await?;

    let mut cover_image = None;
    if let Some(ref url) = novel_info.cover_url {
        match client.image(url).await {
            Ok(image) => cover_image = Some(image),
            Err(error) => {
                error!("Cover image download failed: `{error}`");
            }
        }
    }

    utils::print_novel_info(cover_image, novel_info, &config.converts)?;

    Ok(())
}
