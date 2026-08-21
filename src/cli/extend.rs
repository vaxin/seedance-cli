use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::client::types::{ContentItem, CreateTaskRequest, UrlRef};
use crate::client::ArkClient;
use crate::config::AppConfig;
use crate::core::{tos, upload, video};

use super::common::{self, SubmitOpts};
use super::generate::resolve_prompt;

#[derive(Debug, Args)]
pub struct ExtendArgs {
    /// Source video file(s) to extend (1 for forward/backward, 2-3 for bridging).
    /// Videos longer than min(5s, --duration) are auto-trimmed from the end.
    #[arg(short = 's', long = "source-video", required = true, action = clap::ArgAction::Append)]
    pub source_video: Vec<String>,

    /// Prompt describing the extension content (or @file.txt)
    pub prompt: String,

    // ── Generation params ──

    /// Model: standard (2.0) | fast (2.0) | 2.5 — or a raw Ark model ID.
    /// 2.5 uses the native `extend` task type (no 5s auto-trim, 4-30s)
    #[arg(short, long, default_value = "standard")]
    pub model: String,

    /// Duration in seconds (2.0: 4-15; 2.5: 4-30, or -1 to let the model pick)
    #[arg(short, long, default_value_t = 5, allow_negative_numbers = true)]
    pub duration: i32,

    /// Aspect ratio (forced to "adaptive" on 2.5 native extend)
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

    let spec = super::models::resolve_spec(&args.model);
    let model_id = spec.id.to_string();
    let native_extend = spec.supports_task_types;

    super::models::validate_duration(&spec, args.duration)?;
    super::models::validate_resolution(&spec, &args.resolution)?;

    if args.source_video.is_empty() || args.source_video.len() > 3 {
        anyhow::bail!(
            "must specify 1–3 source videos (got {}). Use -s/--source-video for each.",
            args.source_video.len()
        );
    }

    // Auto-cleanup expired TOS temp files from previous runs (best-effort)
    let _ = tos::cleanup_expired().await;

    let prompt = resolve_prompt(&args.prompt)?;

    let mut content = vec![ContentItem::Text {
        text: prompt.clone(),
    }];

    // On Seedance 2.5 the native `extend` task type accepts the full source
    // clip (2–30s each, 30s total across clips) and requires an adaptive
    // ratio. Older models keep the legacy workaround: trim each clip to
    // min(5s, duration) and chain via the reference video.
    let max_source_duration = if native_extend {
        30 / args.source_video.len().max(1) as i32
    } else {
        args.duration.min(5)
    };
    let mut source_urls: Vec<String> = Vec::new();

    for src in &args.source_video {
        let url = video::prepare_source_video(src, max_source_duration).await?;
        source_urls.push(url.clone());
        content.push(ContentItem::VideoUrl {
            video_url: UrlRef { url },
            role: Some("reference_video".into()),
        });
    }

    for img in &args.image {
        if args.image.len() > spec.max_images {
            anyhow::bail!(
                "max {} image references allowed for this model (got {})",
                spec.max_images,
                args.image.len()
            );
        }
        let url = upload::resolve_file_ref(img)?;
        content.push(ContentItem::ImageUrl {
            image_url: UrlRef { url },
            role: Some("reference_image".into()),
        });
    }

    // 2.5 native extend forces an adaptive ratio (output matches the source).
    let ratio = if native_extend {
        if !args.quiet && args.ratio != "adaptive" {
            eprintln!("note: Seedance 2.5 extend forces ratio=adaptive, ignoring --ratio");
        }
        "adaptive".to_string()
    } else {
        args.ratio.clone()
    };

    let req = CreateTaskRequest {
        model: model_id.clone(),
        content,
        resolution: Some(args.resolution.clone()),
        ratio: Some(ratio),
        duration: Some(args.duration),
        watermark: Some(args.watermark),
        generate_audio: if spec.version >= 25 {
            Some(args.audio_gen)
        } else if args.audio_gen {
            Some(true)
        } else {
            None
        },
        seed: args.seed,
        return_last_frame: if args.return_last_frame {
            Some(true)
        } else {
            None
        },
        callback_url: None,
        tools: None,
        service_tier: None,
        task_type: if native_extend {
            Some("extend".into())
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

    common::submit_and_handle(&client, &req, &prompt, &model_id, opts).await?;

    // Clean up TOS temp files after successful generation (only if we waited)
    if args.wait {
        for url in &source_urls {
            if let Err(e) = tos::delete_file(url).await {
                if !args.quiet {
                    eprintln!("warning: failed to clean up TOS temp file: {e:#}");
                }
            }
        }
    }

    Ok(())
}
