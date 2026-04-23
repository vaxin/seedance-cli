use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::client::types::{ContentItem, CreateTaskRequest, Tool, UrlRef};
use crate::client::ArkClient;
use crate::config::AppConfig;
use crate::core::{downloader, poller, upload};
use crate::store::TaskStore;
use crate::ui;

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// Video description prompt (or @file.txt to read from file)
    pub prompt: String,

    // ── Generation params ──

    /// Model: standard or fast
    #[arg(short, long, default_value = "standard")]
    pub model: String,

    /// Duration in seconds (4-15)
    #[arg(short, long, default_value_t = 5)]
    pub duration: u8,

    /// Aspect ratio
    #[arg(short, long, default_value = "16:9")]
    pub ratio: String,

    /// Resolution
    #[arg(long, default_value = "1080p")]
    pub resolution: String,

    /// Random seed for reproducibility
    #[arg(long)]
    pub seed: Option<u64>,

    /// Add watermark
    #[arg(long, default_value_t = false)]
    pub watermark: bool,

    /// Enable native audio generation
    #[arg(long, default_value_t = false)]
    pub audio_gen: bool,

    /// Return the last frame as an image
    #[arg(long, default_value_t = false)]
    pub return_last_frame: bool,

    /// Webhook callback URL
    #[arg(long)]
    pub callback: Option<String>,

    /// Enable web search tool (text-only input)
    #[arg(long, default_value_t = false)]
    pub web_search: bool,

    /// Service tier: "default" or "flex" (offline inference)
    #[arg(long)]
    pub service_tier: Option<String>,

    // ── Material inputs ──

    /// Image reference (URL or local path, repeatable, max 9)
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub image: Vec<String>,

    /// Video reference (URL or local path, repeatable, max 3)
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub video: Vec<String>,

    /// Audio reference (URL or local path, repeatable, max 3)
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub audio: Vec<String>,

    /// First frame image
    #[arg(long)]
    pub first_frame: Option<String>,

    /// Last frame image
    #[arg(long)]
    pub last_frame: Option<String>,

    // ── Wait & output ──

    /// Wait for task completion and auto-download
    #[arg(short, long, default_value_t = false)]
    pub wait: bool,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Max wait time in seconds (with --wait)
    #[arg(long, default_value_t = 300)]
    pub timeout: u64,

    /// Poll interval in seconds
    #[arg(long, default_value_t = 10)]
    pub poll_interval: u64,

    /// Treat timeout as failure with non-zero exit code (for CI/scripts)
    #[arg(long, default_value_t = false)]
    pub strict: bool,

    // ── Output format ──

    /// Quiet mode: only print task_id or final file path
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// JSON output
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

pub async fn execute(args: GenerateArgs) -> Result<()> {
    let cfg = AppConfig::load()?;
    let api_key = cfg.resolve_api_key()?;
    let client = ArkClient::new(&cfg.base_url, &api_key)?;

    let prompt = resolve_prompt(&args.prompt)?;

    validate_inputs(&args)?;

    let model_id = resolve_model_id(&args.model);

    let mut content = vec![ContentItem::Text {
        text: prompt.clone(),
    }];

    if let Some(ref ff) = args.first_frame {
        let url = upload::resolve_file_ref(ff)?;
        content.push(ContentItem::ImageUrl {
            image_url: UrlRef { url },
            role: Some("first_frame".into()),
        });
    }
    if let Some(ref lf) = args.last_frame {
        let url = upload::resolve_file_ref(lf)?;
        content.push(ContentItem::ImageUrl {
            image_url: UrlRef { url },
            role: Some("last_frame".into()),
        });
    }

    for img in &args.image {
        let url = upload::resolve_file_ref(img)?;
        content.push(ContentItem::ImageUrl {
            image_url: UrlRef { url },
            role: Some("reference_image".into()),
        });
    }
    for vid in &args.video {
        let url = upload::resolve_file_ref(vid)?;
        content.push(ContentItem::VideoUrl {
            video_url: UrlRef { url },
            role: Some("reference_video".into()),
        });
    }
    for aud in &args.audio {
        let url = upload::resolve_file_ref(aud)?;
        content.push(ContentItem::AudioUrl {
            audio_url: UrlRef { url },
            role: Some("reference_audio".into()),
        });
    }

    let tools = if args.web_search {
        Some(vec![Tool {
            tool_type: "web_search".into(),
        }])
    } else {
        None
    };

    let req = CreateTaskRequest {
        model: model_id.clone(),
        content,
        resolution: Some(args.resolution.clone()),
        ratio: Some(args.ratio.clone()),
        duration: Some(args.duration),
        watermark: Some(args.watermark),
        generate_audio: if args.audio_gen { Some(true) } else { None },
        seed: args.seed,
        return_last_frame: if args.return_last_frame {
            Some(true)
        } else {
            None
        },
        callback_url: args.callback.clone(),
        tools,
        service_tier: args.service_tier.clone(),
    };

    let resp = client.create_task(&req).await?;
    let task_id = resp.id.clone();

    if let Ok(store) = TaskStore::open() {
        let _ = store.insert(&task_id, &prompt, &model_id);
    }

    if args.json && !args.wait {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    if !args.quiet {
        ui::print_task_created(&task_id);
    } else {
        println!("{task_id}");
    }

    if !args.wait {
        return Ok(());
    }

    // ── Wait mode ──

    let poll_opts = poller::PollOptions {
        timeout: std::time::Duration::from_secs(args.timeout),
        initial_interval: std::time::Duration::from_secs(args.poll_interval),
        strict: args.strict,
    };

    let result = poller::poll_until_done(&client, &task_id, &poll_opts, args.quiet).await?;

    match result {
        poller::PollResult::Completed(task) => {
            let status = task.task_status();

            if let Ok(store) = TaskStore::open() {
                let _ = store.update_status(
                    &task_id,
                    status.label(),
                    task.resolved_video_url(),
                );
            }

            if args.json {
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
                let output = args
                    .output
                    .unwrap_or_else(|| PathBuf::from(format!("seedance_{task_id}.mp4")));

                downloader::download_video(video_url, &output, args.quiet).await?;

                if let Ok(store) = TaskStore::open() {
                    let _ = store.update_output_path(&task_id, &output.display().to_string());
                }

                if args.quiet {
                    println!("{}", output.display());
                } else {
                    ui::print_downloaded(&output.display().to_string());
                }
            }
        }
        poller::PollResult::TimedOut {
            task_id,
            elapsed,
            ..
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
            } else if !args.quiet {
                ui::print_timeout_hint(&task_id, elapsed.as_secs());
            } else {
                println!("{task_id}");
            }

            if args.strict {
                std::process::exit(2);
            }
        }
    }

    Ok(())
}

pub fn resolve_prompt(input: &str) -> Result<String> {
    if let Some(path) = input.strip_prefix('@') {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read prompt file {path}: {e}"))?;
        Ok(content.trim().to_string())
    } else {
        Ok(input.to_string())
    }
}

pub fn resolve_model_id(model: &str) -> String {
    match model {
        "standard" | "std" => "doubao-seedance-2-0-260128".into(),
        "fast" => "doubao-seedance-2-0-fast-260128".into(),
        other => other.into(), // allow raw model IDs
    }
}

pub fn validate_inputs(args: &GenerateArgs) -> Result<()> {
    if args.duration < 4 || args.duration > 15 {
        anyhow::bail!("duration must be between 4 and 15 seconds");
    }
    if args.image.len() > 9 {
        anyhow::bail!("max 9 image references allowed (Rule of 12)");
    }
    if args.video.len() > 3 {
        anyhow::bail!("max 3 video references allowed (Rule of 12)");
    }
    if args.audio.len() > 3 {
        anyhow::bail!("max 3 audio references allowed (Rule of 12)");
    }
    let total_files = args.image.len()
        + args.video.len()
        + args.audio.len()
        + args.first_frame.as_ref().map(|_| 1).unwrap_or(0)
        + args.last_frame.as_ref().map(|_| 1).unwrap_or(0);
    if total_files > 12 {
        anyhow::bail!("max 12 total files allowed (Rule of 12), got {total_files}");
    }
    Ok(())
}
