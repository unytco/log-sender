//! Check database sizes.

use crate::*;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "k", rename_all = "camelCase")]
enum ReportEntry {
    DbSize {
        #[serde(rename = "t")]
        timestamp: String,

        #[serde(rename = "d")]
        space: String,

        #[serde(rename = "b")]
        total_bytes: String,
    },
}

/// Check database sizes.
pub async fn check_db_size(config: &RuntimeConfig) -> Result<Vec<String>> {
    let mut out = Vec::new();

    for conductor in config.conductor_config_path_list.iter() {
        let conductor = tokio::fs::read_to_string(&conductor).await?;

        #[derive(Debug, serde::Deserialize)]
        struct C {
            data_root_path: std::path::PathBuf,
        }

        let conductor: C =
            serde_yaml::from_str(&conductor).map_err(std::io::Error::other)?;

        // only check dht database for now... it's gossipy : )
        let db_dir = conductor.data_root_path.join("databases").join("dht");

        tracing::trace!(?db_dir);

        out.append(&mut get_sizes(&db_dir).await?);
    }

    Ok(out)
}

async fn get_sizes(dir: &std::path::Path) -> Result<Vec<String>> {
    let mut map: HashMap<String, u64> = HashMap::new();

    let mut dir = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = dir.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }

        let meta = entry.metadata().await?;

        let name = entry
            .file_name()
            .to_string_lossy()
            .trim_end_matches("-shm")
            .trim_end_matches("-wal")
            .to_string();

        *map.entry(name).or_default() += meta.len();
    }

    let now = std::time::SystemTime::UNIX_EPOCH
        .elapsed()
        .expect("system time")
        .as_micros()
        .to_string();

    let mut out = Vec::with_capacity(map.len());

    for (k, v) in map {
        out.push(
            serde_json::to_string(&ReportEntry::DbSize {
                timestamp: now.clone(),
                space: k,
                total_bytes: v.to_string(),
            })
            .map_err(std::io::Error::other)?,
        );
    }

    Ok(out)
}
