use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub api_key: Option<String>,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_resolution")]
    pub resolution: String,
    #[serde(default = "default_ratio")]
    pub ratio: String,
    #[serde(default = "default_duration")]
    pub duration: u8,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

fn default_base_url() -> String {
    "https://ark.cn-beijing.volces.com/api/v3".into()
}
fn default_model() -> String {
    "standard".into()
}
fn default_resolution() -> String {
    "1080p".into()
}
fn default_ratio() -> String {
    "16:9".into()
}
fn default_duration() -> u8 {
    5
}
fn default_output_dir() -> String {
    ".".into()
}

impl AppConfig {
    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("cannot determine config directory")?
            .join("seedance");
        Ok(dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::with_defaults());
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let cfg: AppConfig =
            toml::from_str(&contents).with_context(|| "failed to parse config.toml")?;
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }

    pub fn with_defaults() -> Self {
        Self {
            api_key: None,
            base_url: default_base_url(),
            model: default_model(),
            resolution: default_resolution(),
            ratio: default_ratio(),
            duration: default_duration(),
            output_dir: default_output_dir(),
        }
    }

    /// Resolve API key: CLI arg > env var > config file
    pub fn resolve_api_key(&self) -> Result<String> {
        if let Ok(key) = std::env::var("ARK_API_KEY") {
            if !key.is_empty() {
                return Ok(key);
            }
        }
        self.api_key
            .clone()
            .filter(|k| !k.is_empty())
            .context("API key not configured. Run `seedance config init` or set ARK_API_KEY env var.")
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "api_key" => self.api_key.clone(),
            "base_url" => Some(self.base_url.clone()),
            "model" => Some(self.model.clone()),
            "resolution" => Some(self.resolution.clone()),
            "ratio" => Some(self.ratio.clone()),
            "duration" => Some(self.duration.to_string()),
            "output_dir" => Some(self.output_dir.clone()),
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "api_key" => self.api_key = Some(value.into()),
            "base_url" => self.base_url = value.into(),
            "model" => self.model = value.into(),
            "resolution" => self.resolution = value.into(),
            "ratio" => self.ratio = value.into(),
            "duration" => {
                self.duration = value
                    .parse()
                    .context("duration must be a number between 4 and 15")?
            }
            "output_dir" => self.output_dir = value.into(),
            _ => anyhow::bail!("unknown config key: {key}"),
        }
        Ok(())
    }
}
