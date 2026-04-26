use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::client::types::{ContentItem, CreateTaskRequest, UrlRef};
use crate::client::ArkClient;
use crate::config::AppConfig;
use crate::core::upload;

use super::common::{self, SubmitOpts};
use super::generate::{resolve_model_id, resolve_prompt};

#[derive(Debug, Args)]
pub struct ExtendArgs {
    /// Source task ID(s) to extend (1 for forward/backward, 2-3 for bridging)
    #[arg(required = true, num_args = 1..=3)]
    pub source: Vec<String>,

    /// Prompt describing the extension content (or @file.txt)
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

    /// Enable native audio generation
    #[arg(long, default_value_t = false)]
    pub audio_gen: bool,

    /// Add watermark
    #[arg(long, default_value_t = false)]
    pub watermark: bool,

    /// Return the last frame (useful for chaining further extends)
    #[arg(long, default_value_t = false)]
    pub return_last_frame: bool,

    /// Random seed
    #[arg(long)]
    pub seed: Option<u64>,

    /// Additional image reference (URL, local path, or asset://...)
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub image: Vec<String>,

    // ── Wait & output ──

    /// Wait for task completion and auto-download
    #[arg(short, long, default_value_t = false)]
    pub wait: bool,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Max wait time in seconds
    #[arg(long, default_value_t = 300)]
    pub timeout: u64,

    /// Poll interval in seconds
    #[arg(long, default_value_t = 10)]
    pub poll_interval: u64,

    /// Treat timeout as failure exit code
    #[arg(long, default_value_t = false)]
    pub strict: bool,

    /// Quiet mode
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// JSON output
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

pub async fn execute(args: ExtendArgs) -> Result<()> {
    let cfg = AppConfig::load()?;
    let api_key = cfg.resolve_api_key()?;
    let client = ArkClient::new(&cfg.base_url, &api_key)?;

    if args.duration < 4 || args.duration > 15 {
        anyhow::bail!("duration must be between 4 and 15 seconds");
    }

    let prompt = resolve_prompt(&args.prompt)?;
    let model_id = resolve_model_id(&args.model);

    let mut content = vec![ContentItem::Text {
        text: prompt.clone(),
    }];

    for source_id in &args.source {
        let video_url = common::resolve_source_video_url(&client, source_id).await?;
        content.push(ContentItem::VideoUrl {
            video_url: UrlRef { url: video_url },
            role: Some("reference_video".into()),
        });
    }

    for img in &args.image {
        let url = upload::resolve_file_ref(img)?;
        content.push(ContentItem::ImageUrl {
            image_url: UrlRef { url },
            role: Some("reference_image".into()),
        });
    }

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
        callback_url: None,
        tools: None,
        service_tier: None,
    };

    let opts = SubmitOpts {
        wait: args.wait,
        output: args.output,
        timeout: args.timeout,
        poll_interval: args.poll_interval,
        strict: args.strict,
        quiet: args.quiet,
        json: args.json,
    };

    common::submit_and_handle(&client, &req, &prompt, &model_id, opts).await
}
