use anyhow::Result;
use clap::Subcommand;
use console::style;
use dialoguer::{Input, Select};

use crate::config::AppConfig;

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Interactive setup
    Init,
    /// Get a config value
    Get {
        /// Config key
        key: String,
    },
    /// Set a config value
    Set {
        /// Config key
        key: String,
        /// Config value
        value: String,
    },
    /// Show all config
    Show,
}

pub async fn execute(cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Init => init_interactive().await,
        ConfigCommand::Get { key } => get_key(&key),
        ConfigCommand::Set { key, value } => set_key(&key, &value),
        ConfigCommand::Show => show_all(),
    }
}

async fn init_interactive() -> Result<()> {
    let mut cfg = AppConfig::load().unwrap_or_else(|_| AppConfig::with_defaults());

    eprintln!("{}", style("Seedance CLI Configuration").bold().cyan());
    eprintln!();

    let api_key: String = Input::new()
        .with_prompt("API Key (ARK_API_KEY)")
        .with_initial_text(cfg.api_key.as_deref().unwrap_or(""))
        .allow_empty(true)
        .interact_text()?;
    if !api_key.is_empty() {
        cfg.api_key = Some(api_key);
    }

    let models = vec!["standard", "fast"];
    let default_idx = models
        .iter()
        .position(|m| *m == cfg.model)
        .unwrap_or(0);
    let model_idx = Select::new()
        .with_prompt("Default model")
        .items(&models)
        .default(default_idx)
        .interact()?;
    cfg.model = models[model_idx].into();

    let resolutions = vec!["480p", "720p", "1080p", "2K"];
    let default_idx = resolutions
        .iter()
        .position(|r| *r == cfg.resolution)
        .unwrap_or(2);
    let res_idx = Select::new()
        .with_prompt("Default resolution")
        .items(&resolutions)
        .default(default_idx)
        .interact()?;
    cfg.resolution = resolutions[res_idx].into();

    let ratios = vec!["16:9", "9:16", "4:3", "3:4", "21:9", "1:1"];
    let default_idx = ratios
        .iter()
        .position(|r| *r == cfg.ratio)
        .unwrap_or(0);
    let ratio_idx = Select::new()
        .with_prompt("Default aspect ratio")
        .items(&ratios)
        .default(default_idx)
        .interact()?;
    cfg.ratio = ratios[ratio_idx].into();

    let duration: String = Input::new()
        .with_prompt("Default duration (4-15 seconds)")
        .with_initial_text(cfg.duration.to_string())
        .interact_text()?;
    cfg.duration = duration.parse().unwrap_or(5);

    let output_dir: String = Input::new()
        .with_prompt("Default output directory")
        .with_initial_text(&cfg.output_dir)
        .interact_text()?;
    cfg.output_dir = output_dir;

    cfg.save()?;
    eprintln!();
    eprintln!(
        "{} Config saved to {}",
        style("✓").green().bold(),
        style(AppConfig::config_path()?.display().to_string()).cyan()
    );
    Ok(())
}

fn get_key(key: &str) -> Result<()> {
    let cfg = AppConfig::load()?;
    match cfg.get(key) {
        Some(val) => println!("{val}"),
        None => {
            eprintln!("Unknown config key: {key}");
            eprintln!("Available keys: api_key, base_url, model, resolution, ratio, duration, output_dir");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn set_key(key: &str, value: &str) -> Result<()> {
    let mut cfg = AppConfig::load()?;
    cfg.set(key, value)?;
    cfg.save()?;
    eprintln!(
        "{} {} = {}",
        style("✓").green().bold(),
        key,
        style(value).cyan()
    );
    Ok(())
}

fn show_all() -> Result<()> {
    let cfg = AppConfig::load()?;
    let keys = [
        "api_key",
        "base_url",
        "model",
        "resolution",
        "ratio",
        "duration",
        "output_dir",
    ];
    for key in &keys {
        let val = cfg.get(key).unwrap_or_else(|| "(not set)".into());
        let display_val = if *key == "api_key" {
            match &val {
                v if v.is_empty() || v == "(not set)" => "(not set)".into(),
                v if v.len() > 8 => format!("{}...{}", &v[..4], &v[v.len() - 4..]),
                v => v.clone(),
            }
        } else {
            val
        };
        eprintln!("{:>12} = {}", style(key).bold(), display_val);
    }
    Ok(())
}
