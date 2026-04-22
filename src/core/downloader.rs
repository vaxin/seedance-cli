use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use tokio::io::AsyncWriteExt;

pub async fn download_video(url: &str, output: &Path, quiet: bool) -> Result<()> {
    let resp = reqwest::get(url)
        .await
        .context("failed to start download")?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "download failed with status {}: video URL may have expired (24h limit)",
            resp.status()
        );
    }

    let total = resp.content_length();

    let pb = if !quiet {
        let pb = if let Some(total) = total {
            let pb = ProgressBar::new(total);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("=> "),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} downloaded {bytes}")
                    .unwrap(),
            );
            pb
        };
        Some(pb)
    } else {
        None
    };

    if let Some(parent) = output.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let mut file = tokio::fs::File::create(output)
        .await
        .with_context(|| format!("cannot create {}", output.display()))?;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("error reading download stream")?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        if let Some(pb) = &pb {
            pb.set_position(downloaded);
        }
    }

    file.flush().await?;

    if let Some(pb) = &pb {
        pb.finish_with_message("download complete");
    }

    Ok(())
}
