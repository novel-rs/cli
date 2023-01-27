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

use std::path::Path;

use clap::ValueEnum;
use novel_api::Client;
use strum::AsRefStr;
use url::Url;

#[must_use]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, AsRefStr)]
pub enum Source {
    Sfacg,
    Ciweimao,
}

#[must_use]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Format {
    Pandoc,
    Mdbook,
}

#[must_use]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
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
