use std::path::PathBuf;
use std::sync::Arc;

use clap::Args;
use color_eyre::eyre::Result;
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, CiyuanjiClient, Client, SfacgClient};
use url::Url;

use crate::cmd::Source;
use crate::{utils, LANG_ID, LOCALES};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "sign_command"))]
pub struct Sign {
    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source"))]
    pub source: Source,

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

pub async fn execute(config: Sign) -> Result<()> {
    match config.source {
        Source::Sfacg => {
            let mut client = SfacgClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            utils::log_in(&client, &config.source, config.ignore_keyring).await?;
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

async fn do_execute<T>(client: T, config: Sign) -> Result<()>
where
    T: Client + Send + Sync + 'static,
{
    let client = Arc::new(client);
    super::handle_ctrl_c(&client);

    client.sign_in().await?;

    println!(
        "{} {}",
        config.source.as_ref(),
        LOCALES.lookup(&LANG_ID, "sign_in_successfully")
    );
    println!(
        "{}{}",
        LOCALES.lookup(&LANG_ID, "current_money"),
        client.money().await?
    );

    Ok(())
}
