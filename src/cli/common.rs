use anyhow::Result;
use std::path::PathBuf;

use crate::client::types::{CreateTaskRequest, TaskResponse};
use crate::client::ArkClient;
use crate::core::{downloader, poller};
use crate::store::TaskStore;
use crate::ui;

pub struct SubmitOpts {
    pub wait: bool,
    pub output: Option<PathBuf>,
    pub timeout: u64,
    pub poll_interval: u64,
    pub strict: bool,
    pub quiet: bool,
    pub json: bool,
}

pub async fn submit_and_handle(
    client: &ArkClient,
    req: &CreateTaskRequest,
    prompt: &str,
    model_id: &str,
    opts: SubmitOpts,
) -> Result<()> {
    let resp = client.create_task(req).await?;
    let task_id = resp.id.clone();

    if let Ok(store) = TaskStore::open() {
        let _ = store.insert(&task_id, prompt, model_id);
    }

    if opts.json && !opts.wait {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    if !opts.quiet {
        ui::print_task_created(&task_id);
    } else {
        println!("{task_id}");
    }

    if !opts.wait {
        return Ok(());
    }

    let poll_opts = poller::PollOptions {
        timeout: std::time::Duration::from_secs(opts.timeout),
        initial_interval: std::time::Duration::from_secs(opts.poll_interval),
        strict: opts.strict,
    };

    let result = poller::poll_until_done(client, &task_id, &poll_opts, opts.quiet).await?;

    match result {
        poller::PollResult::Completed(task) => {
            handle_completed(task, &task_id, &opts).await?;
        }
        poller::PollResult::TimedOut {
            task_id,
            elapsed,
            ..
        } => {
            handle_timeout(&task_id, elapsed.as_secs(), &opts);
        }
    }

    Ok(())
}

async fn handle_completed(task: TaskResponse, task_id: &str, opts: &SubmitOpts) -> Result<()> {
    let status = task.task_status();

    if let Ok(store) = TaskStore::open() {
        let _ = store.update_status(task_id, status.label(), task.resolved_video_url());
    }

    if opts.json {
        println!("{}", serde_json::to_string_pretty(&task)?);
        if !status.is_success() {
            std::process::exit(1);
        }
        return Ok(());
    }

    if !status.is_success() {
        let msg = task
            .error
            .as_ref()
            .and_then(|e| e.message.as_deref())
            .unwrap_or("unknown error");
        ui::print_error(&format!("Task {task_id} {status}: {msg}"));
        std::process::exit(1);
    }

    if let Some(video_url) = task.resolved_video_url() {
        let output = opts
            .output
            .clone()
            .unwrap_or_else(|| PathBuf::from(format!("seedance_{task_id}.mp4")));

        downloader::download_video(video_url, &output, opts.quiet).await?;

        if let Ok(store) = TaskStore::open() {
            let _ = store.update_output_path(task_id, &output.display().to_string());
        }

        if opts.quiet {
            println!("{}", output.display());
        } else {
            ui::print_downloaded(&output.display().to_string());
        }
    }

    Ok(())
}

fn handle_timeout(task_id: &str, elapsed_secs: u64, opts: &SubmitOpts) {
    if opts.json {
        println!(
            "{}",
            serde_json::json!({
                "task_id": task_id,
                "status": "timeout",
                "elapsed_secs": elapsed_secs,
            })
        );
    } else if !opts.quiet {
        ui::print_timeout_hint(task_id, elapsed_secs);
    } else {
        println!("{task_id}");
    }

    if opts.strict {
        std::process::exit(2);
    }
}

pub async fn resolve_source_video_url(
    client: &ArkClient,
    task_id: &str,
) -> Result<String> {
    let task = client.query_task(task_id).await?;
    let status = task.task_status();
    if !status.is_success() {
        anyhow::bail!("source task {task_id} is not completed (status: {status})");
    }
    task.resolved_video_url()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("source task {task_id} has no video URL"))
}
