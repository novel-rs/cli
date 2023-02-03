use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Ok, Result};
use clap::{value_parser, Args};
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, Client, Options, SfacgClient, Timing};
use tokio::sync::Semaphore;
use tracing::{info, log::warn};
use url::Url;

use crate::{
    cmd::{Convert, Source},
    utils, LANG_ID, LOCALES,
};

#[must_use]
#[derive(Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "search_command").unwrap())]
pub struct Search {
    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source").unwrap())]
    pub source: Source,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "show_category").unwrap())]
    pub show_category: bool,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "show_tags").unwrap())]
    pub show_tags: bool,

    #[arg(help = LOCALES.lookup(&LANG_ID, "keyword").unwrap())]
    pub keyword: Option<String>,

    #[arg(long, conflicts_with = "keyword",
        help = LOCALES.lookup(&LANG_ID, "min_word_count").unwrap())]
    pub min_word_count: Option<u32>,

    #[arg(long, conflicts_with = "keyword",
        help = LOCALES.lookup(&LANG_ID, "max_word_count").unwrap())]
    pub max_word_count: Option<u32>,

    #[arg(long, conflicts_with = "keyword",
        help = LOCALES.lookup(&LANG_ID, "update_days").unwrap())]
    pub update_days: Option<u8>,

    #[arg(long, conflicts_with = "keyword",
        help = LOCALES.lookup(&LANG_ID, "is_finished").unwrap())]
    pub is_finished: Option<bool>,

    #[arg(long, conflicts_with = "keyword",
        help = LOCALES.lookup(&LANG_ID, "is_vip").unwrap())]
    pub is_vip: Option<bool>,

    #[arg(long, conflicts_with = "keyword",
        help = LOCALES.lookup(&LANG_ID, "category").unwrap())]
    pub category: Option<String>,

    #[arg(long, conflicts_with = "keyword", value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "tags").unwrap())]
    pub tags: Vec<String>,

    #[arg(long, conflicts_with = "keyword", value_delimiter = ',',
    help = LOCALES.lookup(&LANG_ID, "exclude_tags").unwrap())]
    pub exclude_tags: Vec<String>,

    #[arg(long, default_value_t = 10,
      help = LOCALES.lookup(&LANG_ID, "limit").unwrap())]
    pub limit: u8,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").unwrap())]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring").unwrap())]
    pub ignore_keyring: bool,

    #[arg(short, long, default_value_t = 8, value_parser = value_parser!(u8).range(1..=16),
    help = LOCALES.lookup(&LANG_ID, "maximum_concurrency").unwrap())]
    pub maximum_concurrency: u8,

    #[arg(long, num_args = 0..=1, default_missing_value = "http://127.0.0.1:8080",
        help = LOCALES.lookup(&LANG_ID, "proxy").unwrap())]
    pub proxy: Option<Url>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "no_proxy").unwrap())]
    pub no_proxy: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = super::default_cert_path(),
        help = {
            let args = {
                let mut map = HashMap::new();
                map.insert(String::from("cert_path"), super::default_cert_path().into());
                map
            };

            LOCALES.lookup_with_args(&LANG_ID, "cert", &args).unwrap()
        })]
    pub cert: Option<PathBuf>,
}

pub async fn execute(config: Search) -> Result<()> {
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

    info!("Time spent on `search`: {}", timing.elapsed()?);

    Ok(())
}

async fn show_categories<T>(client: T) -> Result<()>
where
    T: Client,
{
    let categories = client.categories().await?;
    let categories = categories
        .iter()
        .map(|category| category.name.to_string())
        .collect::<Vec<String>>()
        .join("，");

    println!("{categories}");

    Ok(())
}

async fn show_tags<T>(client: T) -> Result<()>
where
    T: Client,
{
    let tags = client.tags().await?;
    let tags = tags
        .iter()
        .map(|category| category.name.to_string())
        .collect::<Vec<String>>()
        .join("，");

    println!("{tags}");

    Ok(())
}

async fn do_execute<T>(client: T, config: Search) -> Result<()>
where
    T: Client + Send + Sync + 'static,
{
    if config.source == Source::Ciweimao {
        utils::login(&client, &config.source, config.ignore_keyring).await?;
    }

    if config.show_category {
        show_categories(client).await?;
    } else if config.show_tags {
        show_tags(client).await?;
    } else {
        let client = Arc::new(client);
        super::ctrl_c(&client);

        let mut options = Options::default();

        let mut result = Vec::new();
        let tags = client.tags().await?;
        for name in config.tags {
            match tags.iter().find(|tag| tag.name == name) {
                Some(tag) => result.push(tag),
                None => warn!("not found tag, ignore"),
            }
        }
        if !result.is_empty() {
            options.tags = Some(result);
        }

        let mut result = Vec::new();
        let tags = client.tags().await?;
        for name in config.exclude_tags {
            match tags.iter().find(|tag| tag.name == name) {
                Some(tag) => result.push(tag),
                None => warn!("not found tag, ignore"),
            }
        }
        if !result.is_empty() {
            options.exclude_tags = Some(result);
        }

        if config.min_word_count.is_some() {
            options.word_count = Some(novel_api::WordCountRange::RangeFrom(
                config.min_word_count.unwrap()..,
            ));
        }

        let mut novel_infos = Vec::new();

        let semaphore = Arc::new(Semaphore::new(config.maximum_concurrency as usize));

        let mut page = 0;
        let size = 12;
        loop {
            let mut handles = Vec::new();

            let novel_ids = if config.keyword.is_some() {
                client
                    .search_infos(config.keyword.as_ref().unwrap(), page, size)
                    .await?
            } else {
                client.novels(&options, page, size).await?
            };
            if novel_ids.is_empty() {
                break;
            }

            page += 1;
            if page > 30 {
                warn!("Too many requests, terminated");
                break;
            }

            for novel_id in novel_ids {
                let client = Arc::clone(&client);
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                handles.push(tokio::spawn(async move {
                    let novel_info = utils::novel_info(&client, novel_id).await?;
                    drop(permit);
                    Ok(novel_info)
                }));
            }

            for handle in handles {
                let novel_info = handle.await??;

                if !novel_infos.contains(&novel_info) {
                    novel_infos.push(novel_info);
                }
            }

            if novel_infos.len() >= config.limit as usize {
                break;
            }
        }

        novel_infos.truncate(config.limit as usize);

        utils::print_novel_infos(novel_infos, &config.converts)?;
    }

    Ok(())
}
