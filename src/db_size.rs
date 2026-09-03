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

        // Holochain 0.7 stores every database flat under `databases/` as
        // conductor.db, wasm.db, dht-<dna>.db, p2p-peer-meta-<dna>.db.
        // Only the dht databases are metered... they're gossipy : )
        let db_dir = conductor.data_root_path.join("databases");

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

        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("dht-") {
            continue;
        }

        let meta = entry.metadata().await?;

        let name = name
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn finds_flat_dht_databases_of_holochain_0_7() {
        let dir = tempfile::tempdir().unwrap();
        let databases = dir.path().join("databases");
        std::fs::create_dir_all(&databases).unwrap();
        std::fs::write(databases.join("dht-uhC0kAAAA.db"), vec![0u8; 1024]).unwrap();
        std::fs::write(databases.join("conductor.db"), vec![0u8; 512]).unwrap();
        std::fs::write(databases.join("wasm.db"), vec![0u8; 256]).unwrap();

        let sizes = get_sizes(&databases).await.unwrap();
        assert_eq!(sizes.len(), 1, "only dht-* databases are metered: {sizes:?}");
        assert!(sizes[0].contains("dht-uhC0kAAAA"), "{sizes:?}");
    }
}
