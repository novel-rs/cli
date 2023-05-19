use std::{path::PathBuf, sync::Arc};

use clap::{value_parser, Args};
use color_eyre::eyre::Result;
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, Client, ContentInfo, SfacgClient, VolumeInfos};
use tokio::{
    sync::{RwLock, Semaphore},
    task::JoinHandle,
};
use tracing::{error, info, warn};
use url::Url;

use crate::{
    cmd::{Convert, Format, Source},
    renderer,
    utils::{self, Chapter, Content, Image, Novel, ProgressBar, Volume},
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "download_command").unwrap())]
pub struct Download {
    #[arg(help = LOCALES.lookup(&LANG_ID, "novel_id").unwrap())]
    pub novel_id: u32,

    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source").unwrap())]
    pub source: Source,

    #[arg(short, long, value_enum,
        help = LOCALES.lookup(&LANG_ID, "format").unwrap())]
    pub format: Format,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").unwrap())]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring").unwrap())]
    pub ignore_keyring: bool,

    #[arg(short, long, default_value_t = 4, value_parser = value_parser!(u8).range(1..=8),
        help = LOCALES.lookup(&LANG_ID, "maximum_concurrency").unwrap())]
    pub maximum_concurrency: u8,

    #[arg(long, num_args = 0..=1, default_missing_value = super::DEFAULT_PROXY,
        help = LOCALES.lookup(&LANG_ID, "proxy").unwrap())]
    pub proxy: Option<Url>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "no_proxy").unwrap())]
    pub no_proxy: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = super::default_cert_path(),
        help = super::cert_help_msg())]
    pub cert: Option<PathBuf>,
}

pub async fn execute(config: Download) -> Result<()> {
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

async fn do_execute<T>(client: T, config: Download) -> Result<()>
where
    T: Client + Send + Sync + 'static,
{
    let mut user_info = utils::login(&client, &config.source, config.ignore_keyring).await?;
    if user_info.is_none() {
        user_info = client.user_info().await?;
    }
    println!(
        "{}",
        utils::locales_with_arg("login_msg", "✨", user_info.unwrap().nickname)
    );

    let mut handles = Vec::new();
    let mut novel = download_novel(client, &config, &mut handles).await?;

    for handle in handles {
        handle.await??;
    }

    println!("{}", utils::locales("download_complete_msg", "👌"));

    utils::convert(&mut novel, &config.converts).await?;

    match config.format {
        Format::Pandoc => renderer::generate_pandoc_markdown(novel, &config.converts).await?,
        Format::Mdbook => renderer::generate_mdbook(novel, &config.converts).await?,
    };

    Ok(())
}

async fn download_novel<T>(
    client: T,
    config: &Download,
    handles: &mut Vec<JoinHandle<Result<()>>>,
) -> Result<Novel>
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
        cover_image: Arc::new(RwLock::new(None)),
        volumes: Vec::new(),
    };

    if novel_info.cover_url.is_some() {
        handles.push(download_cover_image(
            &client,
            novel_info.cover_url.unwrap(),
            &novel,
        )?);
    }

    println!(
        "{}",
        utils::locales_with_arg("start_msg", "🚚", &novel.name)
    );
    let volume_infos = client.volume_infos(config.novel_id).await?;

    let pb = ProgressBar::new(chapter_count(&volume_infos))?;
    let semaphore = Arc::new(Semaphore::new(config.maximum_concurrency as usize));
    let image_count = Arc::new(RwLock::new(1));

    let mut exists_can_not_downloaded = false;

    for volume_info in volume_infos {
        let mut volume = Volume {
            title: volume_info.title,
            chapters: Vec::new(),
        };

        for chapter_info in volume_info.chapter_infos {
            if chapter_info.can_download() {
                let chapter = Chapter {
                    title: chapter_info.title.clone(),
                    contents: Arc::new(RwLock::new(Vec::new())),
                };

                let client = Arc::clone(&client);
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let mut pb = pb.clone();
                let contents = Arc::clone(&chapter.contents);
                let image_count = Arc::clone(&image_count);

                handles.push(tokio::spawn(async move {
                    pb.inc(&chapter_info.title);
                    let content_infos = client.content_infos(&chapter_info).await?;
                    drop(permit);

                    for content_info in content_infos {
                        match content_info {
                            ContentInfo::Text(text) => {
                                contents.write().await.push(Content::Text(text))
                            }
                            ContentInfo::Image(url) => match client.image(&url).await {
                                Ok(image) => {
                                    let ext = utils::image_ext(&image);

                                    if ext.is_ok() {
                                        let image_name = format!(
                                            "{}.{}",
                                            utils::num_to_str(*image_count.read().await),
                                            ext.unwrap()
                                        );
                                        *image_count.write().await += 1;

                                        let image = Image {
                                            file_name: image_name,
                                            content: image,
                                        };

                                        contents.write().await.push(Content::Image(image));
                                    } else {
                                        error!("{}, url: {url}", ext.unwrap_err())
                                    }
                                }
                                Err(error) => {
                                    error!("Image download failed: {error}");
                                }
                            },
                        }
                    }

                    Ok(())
                }));

                volume.chapters.push(chapter);
            } else {
                info!(
                    "`{}-{}` can not be downloaded",
                    volume.title, chapter_info.title
                );
                exists_can_not_downloaded = true;
            }
        }

        novel.volumes.push(volume);
    }

    pb.finish();

    if exists_can_not_downloaded {
        warn!("There are chapters that cannot be downloaded");
    }

    Ok(novel)
}

fn download_cover_image<T>(
    client: &Arc<T>,
    url: Url,
    novel: &Novel,
) -> Result<JoinHandle<Result<()>>>
where
    T: Client + Sync + Send + 'static,
{
    let client = Arc::clone(client);
    let cover_image = Arc::clone(&novel.cover_image);

    Ok(tokio::spawn(async move {
        match client.image(&url).await {
            Ok(image) => *cover_image.write().await = Some(image),
            Err(error) => {
                error!("Image download failed: {error}");
            }
        };

        Ok(())
    }))
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
