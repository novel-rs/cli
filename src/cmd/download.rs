use std::{path::PathBuf, sync::Arc};

use clap::{value_parser, Args};
use color_eyre::eyre::{self, bail, Result};
use dashmap::DashMap;
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, CiyuanjiClient, Client, ContentInfo, SfacgClient, VolumeInfos};
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use url::Url;

use crate::{
    cmd::{Convert, Format, Source},
    renderer,
    utils::{self, Chapter, Content, Novel, ProgressBar, Volume},
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "download_command"))]
pub struct Download {
    #[arg(help = LOCALES.lookup(&LANG_ID, "novel_id"))]
    pub novel_id: u32,

    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source"))]
    pub source: Source,

    #[arg(short, long, value_enum,
        help = LOCALES.lookup(&LANG_ID, "format"))]
    pub format: Format,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts"))]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring"))]
    pub ignore_keyring: bool,

    #[arg(short, long, default_value_t = 4, value_parser = value_parser!(u8).range(1..=8),
        help = LOCALES.lookup(&LANG_ID, "maximum_concurrency"))]
    pub maximum_concurrency: u8,

    #[arg(long, num_args = 0..=1, default_missing_value = super::DEFAULT_PROXY,
        help = LOCALES.lookup(&LANG_ID, "proxy"))]
    pub proxy: Option<Url>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "no_proxy"))]
    pub no_proxy: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = super::default_cert_path(),
        help = super::cert_help_msg())]
    pub cert: Option<PathBuf>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "skip_login"))]
    pub skip_login: bool,
}

pub async fn execute(config: Download) -> Result<()> {
    check_skip_login_flag(&config)?;

    match config.source {
        Source::Sfacg => {
            let mut client = SfacgClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            do_execute(client, config).await?;
        }
        Source::Ciweimao => {
            let mut client = CiweimaoClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            do_execute(client, config).await?;
        }
        Source::Ciyuanji => {
            let mut client = CiyuanjiClient::new().await?;
            super::set_options(&mut client, &config.proxy, &config.no_proxy, &config.cert);
            do_execute(client, config).await?;
        }
    }

    Ok(())
}

fn check_skip_login_flag(config: &Download) -> Result<()> {
    if config.skip_login && (config.source == Source::Ciweimao || config.source == Source::Ciyuanji)
    {
        bail!(
            "This source cannot skip login: `{}`",
            config.source.as_ref()
        );
    }

    Ok(())
}

async fn do_execute<T>(client: T, config: Download) -> Result<()>
where
    T: Client + Send + Sync + 'static,
{
    if !config.skip_login {
        if config.source == Source::Ciyuanji {
            utils::log_in_without_password(&client).await?;
        } else {
            utils::log_in(&client, &config.source, config.ignore_keyring).await?;
        }

        let user_info = client.user_info().await?;
        println!(
            "{}",
            utils::locales_with_arg("login_msg", "✨", user_info.nickname)
        );
    }

    let mut novel = download_novel(client, &config).await?;
    println!("{}", utils::locales("download_complete_msg", "👌"));

    utils::convert(&mut novel, &config.converts)?;

    match config.format {
        Format::Pandoc => renderer::generate_pandoc_markdown(novel, &config.converts)?,
        Format::Mdbook => renderer::generate_mdbook(novel, &config.converts).await?,
    };

    Ok(())
}

async fn download_novel<T>(client: T, config: &Download) -> Result<Novel>
where
    T: Client + Send + Sync + 'static,
{
    let client = Arc::new(client);
    super::handle_ctrl_c(&client);

    let novel_info = utils::novel_info(&client, config.novel_id).await?;

    let mut novel = Novel {
        name: novel_info.name,
        author_name: novel_info.author_name,
        introduction: novel_info.introduction,
        cover_image: None,
        volumes: Vec::new(),
    };

    println!(
        "{}",
        utils::locales_with_arg("start_msg", "🚚", &novel.name)
    );

    if novel_info.cover_url.is_some() {
        match client.image(&novel_info.cover_url.unwrap()).await {
            Ok(image) => novel.cover_image = Some(image),
            Err(error) => {
                error!("Cover image download failed: `{error}`");
            }
        };
    }

    let Some(volume_infos) = client.volume_infos(config.novel_id).await? else {
        bail!("Unable to get chapter information");
    };

    let mut handles = Vec::with_capacity(128);
    let pb = ProgressBar::new(chapter_count(&volume_infos))?;
    let semaphore = Arc::new(Semaphore::new(config.maximum_concurrency as usize));
    let chapter_map = Arc::new(DashMap::with_capacity(128));

    let mut exists_can_not_downloaded = false;

    for volume_info in volume_infos {
        novel.volumes.push(Volume {
            title: volume_info.title,
            chapters: Vec::with_capacity(32),
        });

        let volume = novel.volumes.last_mut().unwrap();

        for chapter_info in volume_info.chapter_infos {
            if chapter_info.can_download() {
                volume.chapters.push(Chapter {
                    id: chapter_info.id,
                    title: chapter_info.title.clone(),
                    contents: Vec::new(),
                });

                let client = Arc::clone(&client);
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let mut pb = pb.clone();
                let chapter_map = Arc::clone(&chapter_map);

                handles.push(tokio::spawn(async move {
                    pb.inc(&chapter_info.title);
                    let content_infos = client.content_infos(&chapter_info).await?;
                    drop(permit);

                    let mut contents = Vec::with_capacity(32);
                    for content_info in content_infos {
                        match content_info {
                            ContentInfo::Text(text) => contents.push(Content::Text(text)),
                            ContentInfo::Image(url) => match client.image(&url).await {
                                Ok(image) => {
                                    contents.push(Content::Image(image));
                                }
                                Err(error) => {
                                    error!("Image download failed: `{error}`, url: `{url}`");
                                }
                            },
                        }
                    }

                    chapter_map.insert(chapter_info.id, contents);

                    eyre::Ok(())
                }));
            } else {
                info!(
                    "`{}-{}` can not be downloaded",
                    volume.title, chapter_info.title
                );
                exists_can_not_downloaded = true;
            }
        }
    }

    for handle in handles {
        handle.await??;
    }

    let chapter_map = Arc::into_inner(chapter_map).unwrap();
    for volume in &mut novel.volumes {
        for chapter in &mut volume.chapters {
            if let Some((_, contents)) = chapter_map.remove(&chapter.id) {
                chapter.contents = contents;
            }
        }
    }

    pb.finish();

    if exists_can_not_downloaded {
        warn!("There are chapters that cannot be downloaded");
    }

    Ok(novel)
}

#[must_use]
fn chapter_count(volume_infos: &VolumeInfos) -> u64 {
    let mut count = 0;

    for volume_info in volume_infos {
        for chapter_info in &volume_info.chapter_infos {
            if chapter_info.can_download() {
                count += 1;
            }
        }
    }

    count
}
