use anyhow::{Context, Result};
use base64::Engine;
use std::path::Path;

/// Determine if a string is a URL or local path, and resolve to a URL.
/// Local files are converted to base64 data URIs.
pub fn resolve_file_ref(input: &str) -> Result<String> {
    if input.starts_with("http://") || input.starts_with("https://") {
        return Ok(input.to_string());
    }

    let path = Path::new(input);
    if !path.exists() {
        anyhow::bail!("file not found: {input}");
    }

    let data = std::fs::read(path).with_context(|| format!("failed to read {input}"))?;
    let mime = guess_mime(input);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    Ok(format!("data:{mime};base64,{b64}"))
}

pub fn guess_mime(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".mp4") {
        "video/mp4"
    } else if lower.ends_with(".mov") {
        "video/quicktime"
    } else if lower.ends_with(".mp3") {
        "audio/mpeg"
    } else {
        "application/octet-stream"
    }
}
