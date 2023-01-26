use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use clap::Args;
use fluent_templates::Loader;
use novel_api::{CiweimaoClient, Client, NovelInfo, SfacgClient};
use tracing::log::warn;
use url::Url;

use crate::{
    cmd::{Convert, Source},
    utils, LANG_ID, LOCALES,
};

#[derive(Debug, Args)]
#[command(arg_required_else_help = true,
    about = LOCALES.lookup(&LANG_ID, "search_command").expect("`search_command` does not exists"))]
pub struct Search {
    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "source").expect("`source` does not exists"))]
    pub source: Source,

    #[arg(help = LOCALES.lookup(&LANG_ID, "keywords").expect("`keywords` does not exists"))]
    pub keyword: String,

    #[arg(short, long,
        help = LOCALES.lookup(&LANG_ID, "min_word_count").expect("`min_word_count` does not exists"))]
    pub min_word_count: Option<u32>,

    #[arg(short, long, value_delimiter = ',',
        help = LOCALES.lookup(&LANG_ID, "tags").expect("`tags` does not exists"))]
    pub tags: Option<Vec<String>>,

    #[arg(long, default_value_t = 12,
      help = LOCALES.lookup(&LANG_ID, "limit").expect("`limit` does not exists"))]
    pub limit: u8,

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
            do_execute(client, config).await?
        }
    }

    Ok(())
}

async fn do_execute<T>(client: T, config: Search) -> Result<()>
where
    T: Client,
{
    if config.source == Source::Ciweimao {
        utils::login(&client, config.source, config.ignore_keyring).await?;
    }

    let mut novel_infos = Vec::new();

    let mut page = 0;
    let size = 12;
    loop {
        let novel_ids = client.search_infos(&config.keyword, page, size).await?;
        if novel_ids.is_empty() {
            break;
        }

        page += 1;
        if page > 30 {
            warn!("Too many requests, terminated");
            break;
        }

        for novel_id in novel_ids {
            let novel_info = utils::novel_info(&client, novel_id).await?;

            if !novel_infos.contains(&novel_info) && meet_criteria(&novel_info, &config) {
                novel_infos.push(novel_info);
            }
        }

        if novel_infos.len() >= config.limit as usize {
            break;
        }
    }

    novel_infos.truncate(config.limit as usize);

    utils::print_novel_infos(novel_infos, &config.converts)?;

    Ok(())
}

fn meet_criteria(novel_info: &NovelInfo, config: &Search) -> bool {
    meet_word_count_criteria(novel_info, config) && meet_tags_criteria(novel_info, config)
}

fn meet_word_count_criteria(novel_info: &NovelInfo, config: &Search) -> bool {
    if let Some(min_word_count) = config.min_word_count {
        if let Some(word_count) = novel_info.word_count {
            return word_count >= min_word_count;
        }
    }

    true
}

fn meet_tags_criteria(novel_info: &NovelInfo, config: &Search) -> bool {
    if let Some(ref config_tags) = config.tags {
        if novel_info.tags.is_some() {
            let tags: Vec<String> = novel_info
                .tags
                .as_ref()
                .unwrap()
                .iter()
                .map(|tag| tag.name.to_string())
                .collect();

            for config_tag in config_tags {
                if !tags.contains(config_tag) {
                    return false;
                }
            }
        }
    }

    true
}
