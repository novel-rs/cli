pub mod bookshelf;
pub mod build;
pub mod check;
pub mod completions;
pub mod download;
pub mod info;
pub mod read;
pub mod real_cugan;
pub mod search;
pub mod sign;
pub mod template;
pub mod transform;
pub mod unzip;
pub mod update;
pub mod zip;

use std::{collections::HashMap, path::Path, process, sync::Arc};

use clap::ValueEnum;
use fluent_templates::Loader;
use novel_api::Client;
use strum::AsRefStr;
use tokio::signal;
use tracing::warn;
use url::Url;

use crate::{LANG_ID, LOCALES};

const DEFAULT_PROXY: &str = "http://127.0.0.1:8080";

#[must_use]
#[derive(Clone, PartialEq, ValueEnum, AsRefStr)]
pub enum Source {
    #[strum(serialize = "sfacg")]
    Sfacg,
    #[strum(serialize = "ciweimao")]
    Ciweimao,
    #[strum(serialize = "ciyuanji")]
    Ciyuanji,
}

#[must_use]
#[derive(Clone, PartialEq, ValueEnum, AsRefStr)]
pub enum Format {
    Pandoc,
    Mdbook,
}

#[must_use]
#[derive(Clone, PartialEq, ValueEnum, AsRefStr)]
pub enum Convert {
    S2T,
    T2S,
    JP2T2S,
    CUSTOM,
}

#[inline]
#[must_use]
fn default_cert_path() -> String {
    novel_api::home_dir_path()
        .unwrap()
        .join(".mitmproxy")
        .join("mitmproxy-ca-cert.pem")
        .display()
        .to_string()
}

fn set_options<T, E>(client: &mut T, proxy: &Option<Url>, no_proxy: &bool, cert: &Option<E>)
where
    T: Client,
    E: AsRef<Path>,
{
    if let Some(proxy) = proxy {
        client.proxy(proxy.clone());
    }

    if *no_proxy {
        client.no_proxy();
    }

    if let Some(cert) = cert {
        client.cert(cert.as_ref().to_path_buf())
    }
}

fn handle_ctrl_c<T>(client: &Arc<T>)
where
    T: Client + Send + Sync + 'static,
{
    let client = Arc::clone(client);

    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();

        warn!("Download terminated, login data will be saved");

        client.shutdown().await.unwrap();
        process::exit(128 + libc::SIGINT);
    });
}

fn cert_help_msg() -> String {
    let args = {
        let mut map = HashMap::new();
        map.insert(String::from("cert_path"), default_cert_path().into());
        map
    };

    LOCALES.lookup_with_args(&LANG_ID, "cert", &args)
}
