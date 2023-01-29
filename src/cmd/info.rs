use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use clap::Args;
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, Client, SfacgClient};
use tracing::warn;
use url::Url;

use crate::cmd::{Convert, Source};
use crate::{utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Debug, Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "info_command").expect("`info_command` does not exists"))]
pub struct Info {
    #[arg(help = LOCALES.lookup(&LANG_ID, "novel_id").expect("`novel_id` does not exists"))]
    pub novel_id: u32,

    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source").expect("`source` does not exists"))]
    pub source: Source,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").expect("`converts` does not exists"))]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring").expect("`ignore_keyring` does not exists"))]
    pub ignore_keyring: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = "http://127.0.0.1:8080",
        help = LOCALES.lookup(&LANG_ID, "proxy").expect("`proxy` does not exists"))]
    pub proxy: Option<Url>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "no_proxy").expect("`no_proxy` does not exists"))]
    pub no_proxy: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = super::default_cert_path(),
        help = {
            let args = {
                let mut map = HashMap::new();
                map.insert(String::from("cert_path"), super::default_cert_path().into());
                map
            };

            LOCALES.lookup_with_args(&LANG_ID, "cert", &args).expect("`cert` does not exists")
        })]
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
            do_execute(client, config).await?
        }
    }

    Ok(())
}

async fn do_execute<T>(client: T, config: Info) -> Result<()>
where
    T: Client,
{
    if config.source == Source::Ciweimao {
        utils::login(&client, config.source, config.ignore_keyring).await?;
    }

    let novel_info = utils::novel_info(&client, config.novel_id).await?;

    if let Some(ref url) = novel_info.cover_url {
        match client.image_info(url).await {
            Ok(image) => utils::print_novel_info(Some(image), novel_info, &config.converts)?,
            Err(error) => warn!("{error}"),
        }
    } else {
        utils::print_novel_info(None, novel_info, &config.converts)?;
    }

    Ok(())
}
