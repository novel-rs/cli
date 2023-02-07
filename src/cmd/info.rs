use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use bat::{PagingMode, PrettyPrinter};
use clap::{value_parser, Args};
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, Client, ContentInfo, SfacgClient, Timing};
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

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "read").unwrap())]
    pub read: bool,

    #[arg(long, default_value_t = 10, value_parser = value_parser!(u8).range(1..=16),
    help = LOCALES.lookup(&LANG_ID, "limit").unwrap())]
    pub limit: u8,

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
    if config.read {
        utils::login(&client, &config.source, config.ignore_keyring).await?;
    }

    let client = Arc::new(client);
    super::handle_ctrl_c(&client);

    if config.read {
        let mut count = 0;
        let mut result = String::with_capacity(16384);

        'a: for volume in client.volume_infos(config.novel_id).await? {
            result.push_str(&format!("# {}\n\n", volume.title));

            for chapter in volume.chapter_infos {
                if chapter.can_download() {
                    result.push_str(&format!("## {}\n\n", chapter.title));

                    for content_info in client.content_infos(&chapter).await? {
                        if let ContentInfo::Text(line) = content_info {
                            result.push_str(&line);
                            result.push('\n');
                        }
                    }
                    result.push('\n');

                    count += 1;
                    if count >= config.limit {
                        break 'a;
                    }
                }
            }
        }

        let events = utils::to_events(&result, &config.converts)?;
        let mut buf = String::with_capacity(result.len());
        pulldown_cmark_to_cmark::cmark(events.iter(), &mut buf)?;

        PrettyPrinter::new()
            .input_from_bytes(buf.as_bytes())
            .language("markdown")
            .paging_mode(PagingMode::QuitIfOneScreen)
            .print()?;
    } else {
        let novel_info = utils::novel_info(&client, config.novel_id).await?;

        if let Some(ref url) = novel_info.cover_url {
            match client.image(url).await {
                Ok(image) => utils::print_novel_info(Some(image), novel_info, &config.converts)?,
                Err(error) => warn!("{error}"),
            }
        } else {
            utils::print_novel_info(None, novel_info, &config.converts)?;
        }
    }

    Ok(())
}
