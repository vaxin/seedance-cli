use anyhow::Result;
use clap::Args;

use crate::client::ArkClient;
use crate::config::AppConfig;
use crate::core::poller;
use crate::store::TaskStore;
use crate::ui;

#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Task ID to query
    pub task_id: String,

    /// JSON output
    #[arg(long, default_value_t = false)]
    pub json: bool,

    /// Keep polling until task completes
    #[arg(long, default_value_t = false)]
    pub wait: bool,

    /// Max wait time in seconds (with --wait)
    #[arg(long, default_value_t = 300)]
    pub timeout: u64,
}

pub async fn execute(args: StatusArgs) -> Result<()> {
    let cfg = AppConfig::load()?;
    let api_key = cfg.resolve_api_key()?;
    let client = ArkClient::new(&cfg.base_url, &api_key)?;

    if !args.wait {
        let resp = client.query_task(&args.task_id).await?;

        if let Ok(store) = TaskStore::open() {
            let _ = store.update_status(
                &args.task_id,
                &resp.status,
                resp.resolved_video_url(),
            );
        }

        if args.json {
            println!("{}", serde_json::to_string_pretty(&resp)?);
        } else {
            ui::print_task_status(&args.task_id, &resp.status);
            if let Some(url) = resp.resolved_video_url() {
                eprintln!("  video_url: {url}");
            }
            if let Some(ref err) = resp.error {
                if let Some(ref msg) = err.message {
                    eprintln!("  error: {msg}");
                }
            }
            if let Some(ref usage) = resp.usage {
                if let Some(tokens) = usage.total_tokens {
                    eprintln!("  tokens: {tokens}");
                }
            }
        }
        return Ok(());
    }

    // ── Wait mode ──

    let poll_opts = poller::PollOptions {
        timeout: std::time::Duration::from_secs(args.timeout),
        initial_interval: std::time::Duration::from_secs(10),
        strict: false,
    };

    let result = poller::poll_until_done(&client, &args.task_id, &poll_opts, false).await?;

    match result {
        poller::PollResult::Completed(task) => {
            let status = task.task_status();

            if let Ok(store) = TaskStore::open() {
                let _ = store.update_status(
                    &args.task_id,
                    status.label(),
                    task.resolved_video_url(),
                );
            }

            if args.json {
                println!("{}", serde_json::to_string_pretty(&task)?);
            } else {
                ui::print_task_status(&args.task_id, status.label());
                if let Some(url) = task.resolved_video_url() {
                    eprintln!("  video_url: {url}");
                    eprintln!();
                    eprintln!("Download:");
                    eprintln!("  seedance download {} -o output.mp4", args.task_id);
                }
            }

            if !status.is_success() {
                std::process::exit(1);
            }
        }
        poller::PollResult::TimedOut {
            task_id, elapsed, ..
        } => {
            if args.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "task_id": task_id,
                        "status": "timeout",
                        "elapsed_secs": elapsed.as_secs(),
                    })
                );
            } else {
                ui::print_timeout_hint(&task_id, elapsed.as_secs());
            }
        }
    }

    Ok(())
}
