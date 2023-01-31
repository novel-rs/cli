pub mod build;
pub mod check;
pub mod completions;
pub mod download;
pub mod favorites;
pub mod info;
pub mod real_cugan;
pub mod search;
pub mod transform;
pub mod unzip;
pub mod update;
pub mod zip;

use std::{path::Path, process, sync::Arc};

use clap::ValueEnum;
use novel_api::Client;
use strum::AsRefStr;
use tokio::signal;
use tracing::warn;
use url::Url;

#[must_use]
#[derive(Clone, PartialEq, ValueEnum, AsRefStr)]
pub enum Source {
    #[strum(serialize = "sfacg")]
    Sfacg,
    #[strum(serialize = "ciweimao")]
    Ciweimao,
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

#[must_use]
fn default_cert_path() -> String {
    let mut home_path = novel_api::home_dir_path().unwrap();
    home_path.push(".mitmproxy");
    home_path.push("mitmproxy-ca-cert.pem");

    home_path.display().to_string()
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
        client.cert(cert)
    }
}

fn ctrl_c<T>(client: &Arc<T>)
where
    T: Client + Send + Sync + 'static,
{
    let client = Arc::clone(client);

    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();

        warn!("Download terminated, login data will be saved");

        client.shutdown().unwrap();
        process::exit(128 + libc::SIGINT);
    });
}
