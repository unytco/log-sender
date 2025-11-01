#![deny(missing_docs)]
//! log-sender

use std::io::Result;

pub mod client;
use client::*;

pub mod config;
use config::*;

pub mod crypto;
use crypto::*;

pub mod reader;
use reader::*;

/// Initialize a new log-sender configuration file.
pub async fn initialize(
    config_file: std::path::PathBuf,
    endpoint: String,
    unyt_pub_key: String,
    report_interval_seconds: u64,
) -> Result<()> {
    let url = reqwest::Url::parse(&endpoint).map_err(std::io::Error::other)?;

    let mut config = RuntimeConfigFile::with_init(
        config_file,
        endpoint,
        unyt_pub_key,
        0,
        report_interval_seconds,
    )
    .await?;

    let client = Client::new(url).await?;

    client.health().await?;

    let id = client.drone_registration(&config).await?;

    config.drone_id = id;
    config.write().await?;

    Ok(())
}

/// Run the service checking for report logs and reporting them.
pub async fn run_service(
    config_file: std::path::PathBuf,
    report_path_list: Vec<std::path::PathBuf>,
) -> Result<()> {
    if report_path_list.is_empty() {
        return Err(std::io::Error::other("no report paths specified"));
    }

    let mut config = RuntimeConfigFile::with_load(config_file).await?;

    let url =
        reqwest::Url::parse(&config.endpoint).map_err(std::io::Error::other)?;

    let client = Client::new(url).await?;

    client.health().await?;

    loop {
        tracing::debug!("Running reports..");
        match read_reports(
            &report_path_list,
            config.last_record_timestamp.clone(),
            |proofs| async {
                tracing::info!("Reporting {} proofs..", proofs.len());
                client.metrics(&config, proofs).await
            },
        )
        .await
        {
            Ok(timestamp) => {
                config.last_record_timestamp = timestamp;
                config.write().await?;
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                // ignore, this is a non-fatal error
            }
            Err(err) => {
                eprintln!("Error reading reports: {err:?}");
            }
        }

        tracing::debug!("done.");

        tokio::time::sleep(std::time::Duration::from_secs(
            config.report_interval_seconds,
        ))
        .await;
    }
}

#[cfg(test)]
mod test;
