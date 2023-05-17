use std::{path::PathBuf, sync::Arc};

use anyhow::{bail, Ok, Result};
use clap::{value_parser, Args};
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, Client, Options, SfacgClient, Tag, WordCountRange};
use tokio::sync::Semaphore;
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
        help = LOCALES.lookup(&LANG_ID, "show_categories").unwrap())]
    pub show_categories: bool,

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
    help = LOCALES.lookup(&LANG_ID, "excluded_tags").unwrap())]
    pub excluded_tags: Vec<String>,

    #[arg(long, default_value_t = 16, value_parser = value_parser!(u8).range(1..=100),
      help = LOCALES.lookup(&LANG_ID, "limit").unwrap())]
    pub limit: u8,

    #[arg(short, long, value_enum, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "converts").unwrap())]
    pub converts: Vec<Convert>,

    #[arg(long, default_value_t = false,
        help = LOCALES.lookup(&LANG_ID, "ignore_keyring").unwrap())]
    pub ignore_keyring: bool,

    #[arg(short, long, default_value_t = 8, value_parser = value_parser!(u8).range(1..=8),
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

pub async fn execute(config: Search) -> Result<()> {
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

    Ok(())
}

async fn do_execute<T>(client: T, config: Search) -> Result<()>
where
    T: Client + Send + Sync + 'static,
{
    if config.show_categories {
        let categories = client.categories().await?;
        println!("{}", vec_to_string(categories)?);
    } else if config.show_tags {
        let tags = client.tags().await?;
        println!("{}", vec_to_string(tags)?);
    } else {
        let client = Arc::new(client);
        super::handle_ctrl_c(&client);

        let mut page = 0;
        let size = 10;
        let semaphore = Arc::new(Semaphore::new(config.maximum_concurrency as usize));

        let mut options = None;
        if config.keyword.is_none() {
            // TODO Display options
            options = Some(create_options(&client, &config).await?)
        }

        let mut novel_infos = Vec::new();
        loop {
            let novel_ids = if config.keyword.is_some() {
                client
                    .search_infos(config.keyword.as_ref().unwrap(), page, size)
                    .await?
            } else {
                client.novels(options.as_ref().unwrap(), page, size).await?
            };
            if novel_ids.is_empty() {
                break;
            }

            page += 1;

            let mut handles = Vec::new();
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

async fn create_options<T>(client: &Arc<T>, config: &Search) -> Result<Options>
where
    T: Client,
{
    let mut options = Options {
        is_finished: config.is_finished,
        is_vip: config.is_vip,
        update_days: config.update_days,
        ..Default::default()
    };

    if config.category.is_some() {
        let categories = client.categories().await?;
        let name = config.category.as_ref().unwrap();

        match categories.iter().find(|category| category.name == *name) {
            Some(category) => options.category = Some(category.clone()),
            None => {
                bail!(
                    "The category was not found: `{name}`, all available categories are: `{}`",
                    vec_to_string(categories)?
                );
            }
        }
    }

    if !config.tags.is_empty() {
        options.tags = Some(to_tags(client, &config.tags).await?)
    }

    if !config.excluded_tags.is_empty() {
        options.excluded_tags = Some(to_tags(client, &config.excluded_tags).await?)
    }

    if config.min_word_count.is_some() && config.max_word_count.is_none() {
        options.word_count = Some(WordCountRange::RangeFrom(config.min_word_count.unwrap()..));
    } else if config.min_word_count.is_none() && config.max_word_count.is_some() {
        options.word_count = Some(WordCountRange::RangeTo(..config.max_word_count.unwrap()));
    } else if config.min_word_count.is_some() && config.max_word_count.is_some() {
        options.word_count = Some(WordCountRange::Range(
            config.min_word_count.unwrap()..config.max_word_count.unwrap(),
        ));
    }

    Ok(options)
}

async fn to_tags<T>(client: &Arc<T>, str: &Vec<String>) -> Result<Vec<Tag>>
where
    T: Client,
{
    let mut result = Vec::new();

    let tags = client.tags().await?;
    for name in str {
        match tags.iter().find(|tag| tag.name == *name) {
            Some(tag) => result.push(tag.clone()),
            None => {
                bail!(
                    "The tag was not found: `{name}`, all available tags are: `{}`",
                    vec_to_string(tags)?
                );
            }
        }
    }

    Ok(result)
}

fn vec_to_string<T>(vec: &[T]) -> Result<String>
where
    T: ToString,
{
    let result = vec
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<String>>()
        .join("、");

    Ok(result)
}
