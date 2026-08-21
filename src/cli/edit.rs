use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::client::types::{ContentItem, CreateTaskRequest, UrlRef};
use crate::client::ArkClient;
use crate::config::AppConfig;
use crate::core::upload;

use super::common::{self, SubmitOpts};
use super::generate::resolve_prompt;

#[derive(Debug, Args)]
pub struct EditArgs {
    /// Source task ID of the video to edit
    pub source: String,

    /// Editing instruction prompt (or @file.txt)
    pub prompt: String,

    // ── Generation params ──

    /// Model: standard (2.0) | fast (2.0) | 2.5 — or a raw Ark model ID.
    /// 2.5 uses the native `edit` task type (duration/ratio follow the source)
    #[arg(short, long, default_value = "standard")]
    pub model: String,

    /// Duration in seconds (ignored on 2.5 native edit: follows the source video)
    #[arg(short, long, default_value_t = 5)]
    pub duration: i32,

    /// Aspect ratio (forced to "adaptive" on 2.5 native edit)
    #[arg(short, long, default_value = "16:9")]
    pub ratio: String,

    /// Resolution (2.5 supports 480p/720p only)
    #[arg(long, default_value = "1080p")]
    pub resolution: String,

    /// Enable native audio generation
    #[arg(long, default_value_t = false)]
    pub audio_gen: bool,

    /// Add watermark
    #[arg(long, default_value_t = false)]
    pub watermark: bool,

    /// Random seed
    #[arg(long)]
    pub seed: Option<u64>,

    /// Image reference for replacement/addition (URL, local path, or asset://...)
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub image: Vec<String>,

    /// Audio reference (URL, local path, or asset://...)
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub audio: Vec<String>,

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

pub async fn execute(args: EditArgs) -> Result<()> {
    let cfg = AppConfig::load()?;
    let api_key = cfg.resolve_api_key()?;
    let client = ArkClient::new(&cfg.base_url, &api_key)?;

    let spec = super::models::resolve_spec(&args.model);
    let model_id = spec.id.to_string();
    let native_edit = spec.supports_task_types;

    super::models::validate_resolution(&spec, &args.resolution)?;

    // Seedance 2.5 native edit: duration is always -1 (follows the source
    // video's length/ratio) and the ratio must be adaptive.
    let (duration, ratio) = if native_edit {
        if !args.quiet && args.duration != 5 {
            eprintln!("note: Seedance 2.5 edit ignores --duration (follows the source video)");
        }
        (-1, "adaptive".to_string())
    } else {
        super::models::validate_duration(&spec, args.duration)?;
        (args.duration, args.ratio.clone())
    };

    let prompt = resolve_prompt(&args.prompt)?;

    let video_url = common::resolve_source_video_url(&client, &args.source).await?;

    let mut content = vec![ContentItem::Text {
        text: prompt.clone(),
    }];

    content.push(ContentItem::VideoUrl {
        video_url: UrlRef { url: video_url },
        role: Some("reference_video".into()),
    });

    for img in &args.image {
        let url = upload::resolve_file_ref(img)?;
        content.push(ContentItem::ImageUrl {
            image_url: UrlRef { url },
            role: Some("reference_image".into()),
        });
    }

    for aud in &args.audio {
        let url = upload::resolve_file_ref(aud)?;
        content.push(ContentItem::AudioUrl {
            audio_url: UrlRef { url },
            role: Some("reference_audio".into()),
        });
    }

    let req = CreateTaskRequest {
        model: model_id.clone(),
        content,
        resolution: Some(args.resolution.clone()),
        ratio: Some(ratio),
        duration: Some(duration),
        watermark: Some(args.watermark),
        generate_audio: if spec.version >= 25 {
            Some(args.audio_gen)
        } else if args.audio_gen {
            Some(true)
        } else {
            None
        },
        seed: args.seed,
        return_last_frame: None,
        callback_url: None,
        tools: None,
        service_tier: None,
        task_type: if native_edit {
            Some("edit".into())
        } else {
            None
        },
        output_format: None,
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
