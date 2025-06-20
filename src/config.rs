use crate::Zeroizing;
use color_eyre::eyre;
use serde::Deserialize;
use std::{collections::HashSet, fmt::Debug, net::SocketAddr, path::Path};

#[derive(Deserialize)]
pub struct ApiConfiguration {
    pub panic_key: Zeroizing<String>,
}

#[derive(Deserialize)]
pub struct OidcConfiguration {
    pub url: String,
    pub client_id: String,
    pub authorized_groups: HashSet<String>,
}

#[derive(Deserialize)]
pub struct ServerConfiguration {
    pub addr: SocketAddr,
}

#[derive(Deserialize)]
pub struct Configuration {
    pub api: ApiConfiguration,
    pub oidc: OidcConfiguration,
    pub server: ServerConfiguration,
}

#[instrument]
pub async fn load<P>(path: P) -> eyre::Result<Configuration>
where
    P: AsRef<Path> + Debug,
{
    let data = tokio::fs::read(path).await?;
    let config = serde_norway::from_slice(&data)?;
    Ok(config)
}
