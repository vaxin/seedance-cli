use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::client::types::{ContentItem, CreateTaskRequest, Tool, UrlRef};
use crate::client::ArkClient;
use crate::config::AppConfig;
use crate::core::{tos, upload, video};

use super::common::{self, SubmitOpts};
use super::models::resolve_spec;

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// Video description prompt (or @file.txt to read from file)
    pub prompt: String,

    // ── Generation params ──

    /// Model: standard (2.0) | fast (2.0) | 2.5 — or a raw Ark model ID
    #[arg(short, long, default_value = "standard")]
    pub model: String,

    /// Duration in seconds (2.0: 4-15; 2.5: 4-30, or -1 to let the model pick)
    #[arg(short, long, default_value_t = 5, allow_negative_numbers = true)]
    pub duration: i32,

    /// Aspect ratio (2.5 also accepts "adaptive")
    #[arg(short, long, default_value = "16:9")]
    pub ratio: String,

    /// Resolution (2.5 supports 480p/720p only — no 1080p)
    #[arg(long, default_value = "1080p")]
    pub resolution: String,

    /// Output container: mp4 (default) or mov — Seedance 2.5 only.
    /// mov = H.264 + yuv444p + PCM, better for grading/keying
    #[arg(long)]
    pub output_format: Option<String>,

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

    let spec = resolve_spec(&args.model);
    let model_id = spec.id.to_string();

    validate_inputs(&args, &spec)?;

    // Auto-cleanup expired TOS temp files from previous runs
    let _ = tos::cleanup_expired().await;

    let mut content = vec![ContentItem::Text {
        text: prompt.clone(),
    }];
    let mut video_urls: Vec<String> = Vec::new();

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
        let url = video::upload_video_for_api(vid).await?;
        video_urls.push(url.clone());
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
        // 2.5 defaults generate_audio to true server-side, so send an
        // explicit value to keep the CLI flag deterministic across models.
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
        callback_url: args.callback.clone(),
        tools,
        service_tier: args.service_tier.clone(),
        task_type: None,
        output_format: args.output_format.clone(),
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

    // Clean up TOS temp video files after successful generation
    if args.wait {
        for url in &video_urls {
            if let Err(e) = tos::delete_file(url).await {
                if !args.quiet {
                    eprintln!("warning: failed to clean up TOS temp file: {e:#}");
                }
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
    super::models::resolve_model_id(model)
}

pub fn validate_inputs(args: &GenerateArgs, spec: &super::models::ModelSpec) -> Result<()> {
    super::models::validate_duration(spec, args.duration)?;
    super::models::validate_resolution(spec, &args.resolution)?;

    if let Some(fmt) = &args.output_format {
        if !matches!(fmt.as_str(), "mp4" | "mov") {
            anyhow::bail!("output_format must be mp4 or mov, got {fmt}");
        }
        if !spec.supports_mov {
            anyhow::bail!("output_format is only supported by Seedance 2.5");
        }
    }

    if args.image.len() > spec.max_images {
        anyhow::bail!(
            "max {} image references allowed for this model (got {})",
            spec.max_images,
            args.image.len()
        );
    }
    if args.video.len() > spec.max_videos {
        anyhow::bail!(
            "max {} video references allowed for this model (got {})",
            spec.max_videos,
            args.video.len()
        );
    }
    if args.audio.len() > spec.max_audios {
        anyhow::bail!(
            "max {} audio references allowed for this model (got {})",
            spec.max_audios,
            args.audio.len()
        );
    }
    // 2.5 allows up to 50 total reference assets (30 images + 10 videos + 10 audios).
    let total_files = args.image.len() + args.video.len() + args.audio.len()
        + args.first_frame.as_ref().map(|_| 1).unwrap_or(0)
        + args.last_frame.as_ref().map(|_| 1).unwrap_or(0);
    if spec.version >= 25 {
        if total_files > 50 {
            anyhow::bail!("max 50 total reference assets allowed (Seedance 2.5), got {total_files}");
        }
    } else if total_files > 12 {
        anyhow::bail!("max 12 total files allowed (Rule of 12), got {total_files}");
    }
    Ok(())
}
