//! Utilities for reading report files from disk.

use super::*;

#[derive(serde::Deserialize)]
struct Parse {
    #[serde(default)]
    k: String,
    #[serde(default)]
    t: String,
}

/// Read reports from disk. Returns the new time to ignore before.
pub async fn read_reports<F, C>(
    path_list: &[std::path::PathBuf],
    ignore_before: String,
    mut cb: C,
) -> Result<String>
where
    F: std::future::Future<Output = Result<()>>,
    C: FnMut(Vec<String>) -> F,
{
    let ignore_before: u64 =
        ignore_before.parse().map_err(std::io::Error::other)?;
    let mut max_ignore_before = ignore_before;

    use tokio::io::AsyncBufReadExt;

    let mut proofs = Vec::new();

    for dir in path_list.iter() {
        let mut dir = tokio::fs::read_dir(dir).await?;
        while let Ok(Some(e)) = dir.next_entry().await {
            let f = e.file_name().to_string_lossy().to_string();
            if !f.ends_with(".jsonl") {
                continue;
            }
            let f = tokio::fs::File::open(e.path()).await?;
            let f = tokio::io::BufReader::new(f);
            let mut f = f.lines();

            while let Some(line) = f.next_line().await? {
                let p: Parse = match serde_json::from_str(&line) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                if p.k != "fetchedOps" {
                    continue;
                }

                let t: u64 = match p.t.parse() {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                if t <= ignore_before {
                    continue;
                }

                if t > max_ignore_before {
                    max_ignore_before = t;
                }

                proofs.push(line);

                if proofs.len() >= 100 {
                    cb(std::mem::take(&mut proofs)).await?;
                }
            }
        }
    }

    if !proofs.is_empty() {
        cb(proofs).await?;
    }

    Ok(max_ignore_before.to_string())
}
