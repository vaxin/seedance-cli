//! Model registry: alias → Ark model ID, plus per-model capability profiles.
//!
//! Seedance 2.5 (upstream `doubao-seedance-2-5-260628`) differs from 2.0 in:
//! - duration 4–30s (plus `-1` = model picks the best length)
//! - resolution 480p / 720p only (no 1080p / 4K)
//! - reference assets: up to 50 (30 images + 10 videos + 10 audios)
//! - native task types: `generate` / `edit` / `extend`
//! - output format: mp4 (default) or mov (H.264 + yuv444p + PCM)

use std::fmt;

pub const SEEDANCE_2_0: &str = "doubao-seedance-2-0-260128";
pub const SEEDANCE_2_0_FAST: &str = "doubao-seedance-2-0-fast-260128";
pub const SEEDANCE_2_5: &str = "doubao-seedance-2-5-260628";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSpec {
    /// Canonical Ark model ID (raw IDs pass through unchanged)
    pub id: String,
    /// Major version (20 or 25); drives capability checks. Unknown raw IDs
    /// get 20 (conservative legacy profile).
    pub version: u8,
    /// Allowed duration range in seconds (inclusive)
    pub duration_min: i32,
    pub duration_max: i32,
    /// Whether `-1` (model picks duration) is accepted
    pub allow_auto_duration: bool,
    /// Supported resolutions (empty = accept anything, for unknown raw IDs)
    pub resolutions: &'static [&'static str],
    pub max_images: usize,
    pub max_videos: usize,
    pub max_audios: usize,
    /// Whether the model supports native task_type (edit / extend)
    pub supports_task_types: bool,
    /// Whether mov output_format is supported
    pub supports_mov: bool,
}

fn seedance_2_0_spec(id: impl Into<String>) -> ModelSpec {
    ModelSpec {
        id: id.into(),
        version: 20,
        duration_min: 4,
        duration_max: 15,
        allow_auto_duration: false,
        resolutions: &["480p", "720p", "1080p"],
        max_images: 9,
        max_videos: 3,
        max_audios: 3,
        supports_task_types: false,
        supports_mov: false,
    }
}

fn seedance_2_5_spec(id: impl Into<String>) -> ModelSpec {
    ModelSpec {
        id: id.into(),
        version: 25,
        duration_min: 4,
        duration_max: 30,
        allow_auto_duration: true,
        resolutions: &["480p", "720p"],
        max_images: 30,
        max_videos: 10,
        max_audios: 10,
        supports_task_types: true,
        supports_mov: true,
    }
}

impl fmt::Display for ModelSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.id)
    }
}

/// Resolve a `--model` alias (or pass through a raw Ark model ID).
///
/// Aliases:
/// - `standard` / `std` / `2.0` → Seedance 2.0
/// - `fast`                     → Seedance 2.0 Fast
/// - `2.5` / `seedance-2.5`     → Seedance 2.5
pub fn resolve_model_id(model: &str) -> String {
    resolve_spec(model).id
}

/// Resolve to a full [`ModelSpec`]. Unknown raw IDs get a conservative
/// legacy (2.0-style) profile so users can still point the CLI at new
/// model endpoints by ID.
pub fn resolve_spec(model: &str) -> ModelSpec {
    match model.trim().to_lowercase().as_str() {
        "standard" | "std" | "2.0" | "v2.0" | "seedance-2.0" | "seedance-2-0" => {
            seedance_2_0_spec(SEEDANCE_2_0)
        }
        "fast" => seedance_2_0_spec(SEEDANCE_2_0_FAST),
        "2.5" | "v2.5" | "seedance-2.5" | "seedance-2-5" => seedance_2_5_spec(SEEDANCE_2_5),
        SEEDANCE_2_0 => seedance_2_0_spec(SEEDANCE_2_0),
        SEEDANCE_2_0_FAST => seedance_2_0_spec(SEEDANCE_2_0_FAST),
        SEEDANCE_2_5 => seedance_2_5_spec(SEEDANCE_2_5),
        other => seedance_2_0_spec(other),
    }
}

/// Look up the spec for a canonical model ID; unknown IDs fall back to a
/// conservative legacy profile with the raw ID preserved.
pub fn spec_for_id(id: &str) -> ModelSpec {
    match id {
        SEEDANCE_2_0 => seedance_2_0_spec(SEEDANCE_2_0),
        SEEDANCE_2_0_FAST => seedance_2_0_spec(SEEDANCE_2_0_FAST),
        SEEDANCE_2_5 => seedance_2_5_spec(SEEDANCE_2_5),
        other => seedance_2_0_spec(other),
    }
}

/// Validate a duration value against the model's capability profile.
/// `-1` means "let the model pick" (Seedance 2.5 only).
pub fn validate_duration(spec: &ModelSpec, duration: i32) -> anyhow::Result<()> {
    if duration == -1 {
        if spec.allow_auto_duration {
            Ok(())
        } else {
            anyhow::bail!("duration -1 (auto) is only supported by Seedance 2.5");
        }
    } else if duration < spec.duration_min || duration > spec.duration_max {
        anyhow::bail!(
            "duration must be between {} and {} seconds for this model (got {})",
            spec.duration_min,
            spec.duration_max,
            duration
        );
    } else {
        Ok(())
    }
}

/// Validate a resolution string against the model's capability profile.
pub fn validate_resolution(spec: &ModelSpec, resolution: &str) -> anyhow::Result<()> {
    if spec.resolutions.is_empty() || spec.resolutions.contains(&resolution) {
        Ok(())
    } else {
        anyhow::bail!(
            "resolution {resolution} is not supported by this model — allowed: {}",
            spec.resolutions.join(", ")
        );
    }
}
