//! Configuration types.

use super::*;

/// Runtime configuration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

    /// Report interval seconds.
    pub report_interval_seconds: u64,

    /// Last record timestamp sent.
    pub last_record_timestamp: String,
}

impl RuntimeConfig {
    /// Create a new runtime configuration instance.
    pub fn with_init(
        endpoint: String,
        drone_pub_key: String,
        drone_sec_key: String,
        unyt_pub_key: String,
        drone_id: u64,
        report_interval_seconds: u64,
    ) -> Self {
        Self {
            endpoint,
            drone_pub_key,
            drone_sec_key,
            unyt_pub_key,
            drone_id,
            report_interval_seconds,
            last_record_timestamp: "0".into(),
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

impl From<RuntimeConfigFile> for RuntimeConfig {
    fn from(config: RuntimeConfigFile) -> Self {
        config.config
    }
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
        report_interval_seconds: u64,
    ) -> Result<Self> {
        let (rt_drone_pub_key, mut rt_drone_sec_key) =
            generate_keypair().await?;
        rt_drone_sec_key = rt_drone_sec_key.precompute().await?;

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
            report_interval_seconds,
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

    /// Load a runtime config from disk.
    pub async fn with_load(file: std::path::PathBuf) -> Result<Self> {
        use tokio::io::AsyncReadExt;

        let path = file.clone();

        let mut file = tokio::task::spawn_blocking(move || {
            use fs2::FileExt;
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(file)?;
            file.try_lock_exclusive()?;
            std::io::Result::Ok(tokio::fs::File::from_std(file))
        })
        .await??;

        let mut config = String::new();
        file.read_to_string(&mut config).await?;
        let config: RuntimeConfig = serde_json::from_str(&config)?;

        let mut rt_drone_sec_key =
            SecKey::decode(config.drone_sec_key.as_bytes())?;
        rt_drone_sec_key = rt_drone_sec_key.precompute().await?;

        Ok(Self {
            config,
            file,
            path,
            rt_drone_sec_key,
        })
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
