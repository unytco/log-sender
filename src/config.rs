//! Configuration types.

use super::*;

/// Runtime configuration.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
    /// Log collector endpoint.
    pub endpoint: String,

    /// Drone public key.
    pub drone_pub_key: String,

    /// Drone secret key.
    pub drone_sec_key: String,

    /// Unyt public key.
    pub unyt_pub_key: String,

    /// Drone id.
    pub drone_id: u64,

    /// Last record timestamp sent.
    pub last_record_timestamp: f64,
}

impl RuntimeConfig {
    /// Create a new runtime configuration instance.
    pub fn with_init(
        endpoint: String,
        drone_pub_key: String,
        drone_sec_key: String,
        unyt_pub_key: String,
        drone_id: u64,
    ) -> Self {
        Self {
            endpoint,
            drone_pub_key,
            drone_sec_key,
            unyt_pub_key,
            drone_id,
            last_record_timestamp: 0.0,
        }
    }
}

/// Runtime configuration file with advisory locking.
pub struct RuntimeConfigFile {
    config: RuntimeConfig,
    file: tokio::fs::File,
    path: std::path::PathBuf,
    pub(crate) rt_drone_sec_key: SecKey,
}

impl std::ops::Deref for RuntimeConfigFile {
    type Target = RuntimeConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl std::ops::DerefMut for RuntimeConfigFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

impl RuntimeConfigFile {
    /// Initialize a new config file.
    pub async fn with_init(
        file: std::path::PathBuf,
        endpoint: String,
        unyt_pub_key: String,
        drone_id: u64,
    ) -> Result<Self> {
        let (rt_drone_pub_key, rt_drone_sec_key) = generate_keypair().await?;

        let path = file.clone();
        let file = tokio::task::spawn_blocking(move || {
            use fs2::FileExt;
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(file)?;
            file.try_lock_exclusive()?;
            std::io::Result::Ok(tokio::fs::File::from_std(file))
        })
        .await??;

        let config = RuntimeConfig::with_init(
            endpoint,
            rt_drone_pub_key.encode()?,
            rt_drone_sec_key.encode()?,
            unyt_pub_key,
            drone_id,
        );

        let mut this = Self {
            config,
            file,
            path,
            rt_drone_sec_key,
        };

        this.write().await?;

        Ok(this)
    }

    /// Get the path of the file on-disk.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Write the config to the file.
    pub async fn write(&mut self) -> Result<()> {
        use tokio::io::{AsyncSeekExt, AsyncWriteExt};
        let data = serde_json::to_string_pretty(&self.config)?;
        self.file.rewind().await?;
        self.file.set_len(data.len() as u64).await?;
        self.file.write_all(data.as_bytes()).await?;
        self.file.flush().await?;
        Ok(())
    }
}
