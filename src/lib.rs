#![deny(missing_docs)]
//! log-sender

use std::io::Result;

pub mod client;
use client::*;

pub mod config;
use config::*;

pub mod crypto;
use crypto::*;

/// Initialize a new log-sender configuration file.
pub async fn initialize(
    config_file: std::path::PathBuf,
    endpoint: String,
    unyt_pub_key: String,
) -> Result<()> {
    let url = reqwest::Url::parse(&endpoint).map_err(std::io::Error::other)?;

    let mut config =
        RuntimeConfigFile::with_init(config_file, endpoint, unyt_pub_key, 0)
            .await?;

    let client = Client::new(url).await?;

    client.health().await?;

    let id = client.drone_registration(&config).await?;

    config.drone_id = id;
    config.write().await?;

    // TODO remove this test
    client
        .metrics(&config, vec!["test1".into(), "test2".into()])
        .await?;

    Ok(())
}

#[cfg(test)]
mod test;
