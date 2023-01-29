use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::{value_parser, Args};
use console::{Alignment, Emoji};
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, Client, ContentInfo, SfacgClient, UserInfo, VolumeInfos};
use tokio::{
    sync::{RwLock, Semaphore},
    task::JoinHandle,
};
use tracing::{info, warn};
use url::Url;

use crate::{
    cmd::{Convert, Format, Source},
    renderer,
    utils::{self, Chapter, Content, Image, Novel, ProgressBar, Volume},
    LANG_ID, LOCALES,
};

#[must_use]
#[derive(Debug, Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "download_command").expect("`download_command` does not exists"))]
pub struct Download {
    #[arg(help = LOCALES.lookup(&LANG_ID, "novel_id").expect("`novel_id` does not exists"))]
    pub novel_id: u32,

    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source").expect("`source` does not exists"))]
    pub source: Source,

    #[arg(short, long, value_enum,
        help = LOCALES.lookup(&LANG_ID, "format").expect("`format` does not exists"))]
    pub format: Format,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").expect("`converts` does not exists"))]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring").expect("`ignore_keyring` does not exists"))]
    pub ignore_keyring: bool,

    #[arg(short, long, default_value_t = 4, value_parser = value_parser!(u8).range(1..=16),
        help = LOCALES.lookup(&LANG_ID, "maximum_concurrency").expect("`maximum_concurrency` does not exists"))]
    pub maximum_concurrency: u8,

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
    let mut user_info = utils::login(&client, config.source, config.ignore_keyring).await?;
    if user_info.is_none() {
        user_info = client.user_info().await?;
    }

    let user_info = user_info.unwrap();
    print_login_msg(user_info);

    let mut handles = Vec::new();
    let mut novel = download_novel(client, &config, &mut handles).await?;

    for handle in handles {
        handle.await??;
    }

    print_completed_msg();

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
    let novel_info = utils::novel_info(&client, config.novel_id).await?;

    let mut novel = Novel {
        name: novel_info.name,
        author_name: novel_info.author_name,
        introduction: novel_info.introduction,
        cover_image: Arc::new(RwLock::new(None)),
        volumes: Vec::new(),
    };

    let client = Arc::new(client);
    if novel_info.cover_url.is_some() {
        handles.push(cover_image(&client, novel_info.cover_url.unwrap(), &novel)?);
    }

    print_start_msg(&novel.name);
    let volume_infos = client.volume_infos(config.novel_id).await?;

    let pb = Arc::new(parking_lot::RwLock::new(ProgressBar::new(
        chapter_count(&volume_infos) as usize,
    )));
    let semaphore = Arc::new(Semaphore::new(config.maximum_concurrency as usize));
    let image_count = Arc::new(RwLock::new(1));

    let mut exists_cannot_downloaded = false;

    for volume_info in volume_infos {
        let mut volume = Volume {
            title: volume_info.title,
            chapters: Vec::new(),
        };

        for chapter_info in volume_info.chapter_infos {
            if novel_api::is_some_and(chapter_info.accessible, |x| x)
                && novel_api::is_some_and(chapter_info.is_valid, |x| x)
            {
                let chapter = Chapter {
                    title: chapter_info.title.clone(),
                    contents: Arc::new(RwLock::new(Vec::new())),
                };

                let client = Arc::clone(&client);
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let pb = Arc::clone(&pb);
                let contents = Arc::clone(&chapter.contents);
                let image_count = Arc::clone(&image_count);

                handles.push(tokio::spawn(async move {
                    pb.write().inc(&chapter_info.title);
                    let content_infos = client.content_infos(&chapter_info).await?;
                    drop(permit);

                    for content_info in content_infos {
                        match content_info {
                            ContentInfo::Text(text) => {
                                contents.write().await.push(Content::Text(text))
                            }
                            ContentInfo::Image(url) => match client.image_info(&url).await {
                                Ok(image) => {
                                    let image_name = format!(
                                        "{}.{}",
                                        utils::num_to_str(*image_count.read().await),
                                        utils::image_ext(&image)
                                    );
                                    *image_count.write().await += 1;

                                    let image = Image {
                                        file_name: image_name,
                                        content: image,
                                    };

                                    contents.write().await.push(Content::Image(image));
                                }
                                Err(error) => {
                                    warn!("{error}");
                                }
                            },
                        }
                    }

                    Ok(())
                }));

                volume.chapters.push(chapter);
            } else {
                info!("`{}` can not be downloaded", chapter_info.title);
                exists_cannot_downloaded = true;
            }
        }

        novel.volumes.push(volume);
    }

    pb.write().finish();

    if exists_cannot_downloaded {
        warn!("There are chapters that cannot be downloaded");
    }

    Ok(novel)
}

fn cover_image<T>(client: &Arc<T>, url: Url, novel: &Novel) -> Result<JoinHandle<Result<()>>>
where
    T: Client + Sync + Send + 'static,
{
    let client = Arc::clone(client);
    let cover_image = Arc::clone(&novel.cover_image);

    Ok(tokio::spawn(async move {
        match client.image_info(&url).await {
            Ok(image) => *cover_image.write().await = Some(image),
            Err(error) => {
                warn!("{error}");
            }
        };

        Ok(())
    }))
}

#[must_use]
fn chapter_count(volume_infos: &VolumeInfos) -> u16 {
    let mut count = 0;

    for volume_info in volume_infos {
        for chapter_info in &volume_info.chapter_infos {
            if novel_api::is_some_and(chapter_info.accessible, |x| x)
                && novel_api::is_some_and(chapter_info.is_valid, |x| x)
            {
                count += 1;
            }
        }
    }

    count
}

fn print_login_msg(user_info: UserInfo) {
    let args = {
        let emoji = Emoji("✨", ">").to_string();
        let emoji = console::pad_str(&emoji, 2, Alignment::Left, None);

        let mut map = HashMap::new();
        map.insert(String::from("emoji"), emoji.to_string().into());
        map.insert(String::from("name"), user_info.nickname.into());
        map
    };

    println!(
        "{}",
        LOCALES
            .lookup_with_args(&LANG_ID, "login_msg", &args)
            .expect("`login_msg` does not exists")
    );
}

fn print_start_msg<T>(novel_name: T)
where
    T: AsRef<str>,
{
    let args = {
        let emoji = Emoji("🚚", ">").to_string();
        let emoji = console::pad_str(&emoji, 2, Alignment::Left, None);

        let mut map = HashMap::new();
        map.insert(String::from("emoji"), emoji.to_string().into());
        map.insert(String::from("name"), novel_name.as_ref().into());
        map
    };

    println!(
        "{}",
        LOCALES
            .lookup_with_args(&LANG_ID, "start_msg", &args)
            .expect("`start_msg` does not exists")
    );
}

fn print_completed_msg() {
    let args = {
        let emoji = Emoji("✔️", ">").to_string();
        let emoji = console::pad_str(&emoji, 2, Alignment::Left, None);

        let mut map = HashMap::new();
        map.insert(String::from("emoji"), emoji.to_string().into());
        map
    };

    println!(
        "{}",
        LOCALES
            .lookup_with_args(&LANG_ID, "download_complete_msg", &args)
            .expect("`download_complete_msg` does not exists")
    );
}
