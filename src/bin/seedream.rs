//! `seedream` — CLI for Seedream image generation on Volcengine Ark.
//!
//! Shares the same crate as `seedance`: config layout, TOS upload, and the
//! Ark HTTP client are all reused. Image generation uses the synchronous
//! POST /images/generations endpoint, so there is no task polling.

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use console::style;
use std::path::{Path, PathBuf};

use seedance_cli::client::types::{ImageGenRequest, SequentialImageGeneration};
use seedance_cli::client::ArkClient;
use seedance_cli::config::AppConfig;
use seedance_cli::core::downloader;

/// Ark base64 data-URI images must stay under 10MB; larger local files are
/// uploaded to TOS instead (requires TOS_* env vars).
const BASE64_LIMIT: u64 = 10 * 1024 * 1024;

pub const SEEDREAM_4_0: &str = "doubao-seedream-4-0-250828";
pub const SEEDREAM_5_0: &str = "doubao-seedream-5-0-260128";
/// Upstream ID not yet confirmed on Ark — verify in the console before use.
pub const SEEDREAM_5_0_PRO: &str = "doubao-seedream-5-0-pro";

const APP: &str = "seedream";

#[derive(Parser)]
#[command(
    name = "seedream",
    about = "CLI for Seedream image generation on Volcengine Ark",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an image (T2I / I2I / editing / multi-image fusion)
    Generate(GenerateArgs),

    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Args)]
pub struct GenerateArgs {
    /// Image description prompt (or @file.txt to read from file)
    pub prompt: String,

    /// Model: standard (4.0) | 5.0 | pro — or a raw Ark model ID
    #[arg(short, long, default_value = "standard")]
    pub model: String,

    /// Reference image (URL, local path, or asset://...) for i2i / editing /
    /// fusion. Up to 10 references.
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub image: Vec<String>,

    /// Output size: "2048x2048", "1K", "2K", "4K", or "adaptive" (4.0+)
    #[arg(long)]
    pub size: Option<String>,

    /// Output file path (default: seedream-<timestamp>.png in cwd)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Add watermark
    #[arg(long, default_value_t = false)]
    pub watermark: bool,

    /// Random seed
    #[arg(long)]
    pub seed: Option<u64>,

    /// Generate a sequential image group (2-4 images, Seedream 4.0+)
    #[arg(long)]
    pub sequential: Option<u8>,

    /// Response format from the API: url (default) or b64_json
    #[arg(long)]
    pub response_format: Option<String>,

    /// Quiet mode (only print the output file path)
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// JSON output (print the raw API response)
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Set a config value
    Set { key: String, value: String },
    /// Get a config value
    Get { key: String },
    /// Show all config
    Show,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Generate(args) => generate(args).await,
        Commands::Config { command } => config(command),
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}

fn resolve_model(model: &str) -> String {
    match model.trim().to_lowercase().as_str() {
        "standard" | "std" | "4.0" | "4" | "seedream-4.0" | "seedream-4-0" => SEEDREAM_4_0.into(),
        "5.0" | "5" | "seedream-5.0" | "seedream-5-0" => SEEDREAM_5_0.into(),
        "pro" | "5.0-pro" | "seedream-5.0-pro" => SEEDREAM_5_0_PRO.into(),
        other => other.into(), // allow raw model IDs
    }
}

fn resolve_prompt(input: &str) -> Result<String> {
    if let Some(path) = input.strip_prefix('@') {
        std::fs::read_to_string(path).with_context(|| format!("failed to read {path}"))
    } else {
        Ok(input.to_string())
    }
}

/// Resolve an image reference to something the API accepts:
/// URLs pass through; local files become base64 data URIs when small
/// enough, or are uploaded to TOS when they exceed the 10MB base64 limit.
async fn resolve_image_ref(input: &str) -> Result<String> {
    if input.starts_with("http://") || input.starts_with("https://") || input.starts_with("asset://")
    {
        return Ok(input.to_string());
    }

    let path = Path::new(input);
    if !path.exists() {
        anyhow::bail!("file not found: {input}");
    }

    let size = path.metadata()?.len();
    if size <= BASE64_LIMIT {
        seedance_cli::core::upload::resolve_file_ref(input)
    } else {
        if !seedance_cli::core::tos::is_configured() {
            anyhow::bail!(
                "local image {input} is {}MB, over the 10MB inline base64 limit — \
                 set TOS_ACCESS_KEY/TOS_SECRET_KEY/TOS_BUCKET to upload it via TOS",
                size / 1024 / 1024
            );
        }
        seedance_cli::core::tos::upload_file(input).await
    }
}

async fn generate(args: GenerateArgs) -> Result<()> {
    if args.image.len() > 10 {
        anyhow::bail!("max 10 reference images allowed (got {})", args.image.len());
    }
    if let Some(n) = args.sequential {
        if !(2..=4).contains(&n) {
            anyhow::bail!("--sequential must be between 2 and 4");
        }
    }
    if let Some(fmt) = &args.response_format {
        if !matches!(fmt.as_str(), "url" | "b64_json") {
            anyhow::bail!("--response-format must be url or b64_json, got {fmt}");
        }
    }

    let cfg = AppConfig::load_for(APP)?;
    let api_key = cfg.resolve_api_key()?;
    let client = ArkClient::new(&cfg.base_url, &api_key)?;

    let prompt = resolve_prompt(&args.prompt)?;
    let model_id = resolve_model(&args.model);

    let mut images = Vec::new();
    for img in &args.image {
        images.push(resolve_image_ref(img).await?);
    }

    let req = ImageGenRequest {
        model: model_id,
        prompt: prompt.clone(),
        image: images,
        size: args.size.clone(),
        response_format: args.response_format.clone(),
        watermark: Some(args.watermark),
        seed: args.seed,
        sequential_image_generation: args.sequential.map(|n| SequentialImageGeneration {
            max_images: n,
            enable_default_prompt: None,
        }),
    };

    if !args.quiet {
        eprintln!(
            "{}",
            style(format!("Generating image with {req_model}…", req_model = req.model)).cyan()
        );
    }

    let resp = client.generate_image(&req).await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
    }

    if resp.data.is_empty() {
        anyhow::bail!("API returned no images");
    }

    // Save every returned image. Without an explicit --output, files land
    // next to the first reference image's directory (or cwd for T2I).
    // The extension of default-named files is sniffed from the image bytes
    // (Seedream typically returns JPEG).
    let base_dir: PathBuf = args
        .image
        .first()
        .and_then(|r| {
            let p = Path::new(r);
            if p.exists() {
                p.parent().map(|d| d.to_path_buf())
            } else {
                None
            }
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let multiple = resp.data.len() > 1;

    for (idx, item) in resp.data.iter().enumerate() {
        // Fetch the bytes first (download or decode) so we can sniff the type.
        let bytes: Vec<u8> = if let Some(b64) = &item.b64_json {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.decode(b64)?
        } else if let Some(url) = &item.url {
            let tmp = std::env::temp_dir().join(format!(
                "seedream-dl-{stamp}-{idx:02}"
            ));
            downloader::download_video(url, &tmp, args.quiet).await?;
            let bytes = std::fs::read(&tmp)?;
            let _ = std::fs::remove_file(&tmp);
            bytes
        } else {
            anyhow::bail!("image item has neither url nor b64_json");
        };

        let sniffed_ext = image_ext(&bytes);
        let default_name = if multiple {
            format!("seedream-{stamp}-{idx:02}.{sniffed_ext}")
        } else {
            format!("seedream-{stamp}.{sniffed_ext}")
        };

        let out_path = match (&args.output, multiple) {
            (Some(p), false) => p.clone(),
            (Some(p), true) => {
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("png");
                p.with_extension(format!("{idx:02}.{ext}"))
            }
            (None, _) => base_dir.join(default_name),
        };

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&out_path, &bytes)?;

        if args.quiet {
            println!("{}", out_path.display());
        } else {
            println!(
                "{} {}",
                style("saved:").green().bold(),
                out_path.display()
            );
        }
    }

    Ok(())
}

/// Guess the image file extension from magic bytes.
fn image_ext(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "webp"
    } else {
        "png"
    }
}

fn config(cmd: ConfigCommand) -> Result<()> {
    let mut cfg = AppConfig::load_for(APP)?;
    match cmd {
        ConfigCommand::Set { key, value } => {
            let known = [
                "api_key",
                "base_url",
                "model",
                "resolution",
                "ratio",
                "duration",
                "output_dir",
            ];
            if !known.contains(&key.as_str()) {
                anyhow::bail!(
                    "unknown config key: {key} (valid: {})",
                    known.join(", ")
                );
            }
            // reuse the same set() validation as the seedance CLI
            cfg.set(&key, &value)?;
            cfg.save_for(APP)?;
            println!("{key} = {value}");
            Ok(())
        }
        ConfigCommand::Get { key } => match cfg.get(&key) {
            Some(v) => {
                println!("{v}");
                Ok(())
            }
            None => anyhow::bail!("config key not set: {key}"),
        },
        ConfigCommand::Show => {
            println!("{}", toml::to_string_pretty(&cfg)?);
            Ok(())
        }
    }
}
