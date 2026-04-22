use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::client::ArkClient;
use crate::config::AppConfig;
use crate::core::downloader;
use crate::store::TaskStore;
use crate::ui;

#[derive(Debug, Args)]
pub struct DownloadArgs {
    /// Task ID to download
    pub task_id: String,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub async fn execute(args: DownloadArgs) -> Result<()> {
    let cfg = AppConfig::load()?;
    let api_key = cfg.resolve_api_key()?;
    let client = ArkClient::new(&cfg.base_url, &api_key)?;

    let resp = client.query_task(&args.task_id).await?;
    let status = resp.task_status();

    if !status.is_success() {
        ui::print_error(&format!(
            "Task {} is not completed (status: {}). Cannot download yet.",
            args.task_id, status
        ));
        if !status.is_terminal() {
            eprintln!("  Try: seedance status {} --wait", args.task_id);
        }
        std::process::exit(1);
    }

    let video_url = resp
        .resolved_video_url()
        .ok_or_else(|| anyhow::anyhow!("task succeeded but no video URL returned"))?;

    let output = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("seedance_{}.mp4", args.task_id)));

    downloader::download_video(video_url, &output, false).await?;

    if let Ok(store) = TaskStore::open() {
        let _ = store.update_output_path(&args.task_id, &output.display().to_string());
    }

    ui::print_downloaded(&output.display().to_string());
    Ok(())
}
