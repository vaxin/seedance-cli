use anyhow::{Context, Result};
use std::process::Command;

use super::tos;

/// Probe the duration of a video file in seconds using ffprobe.
pub fn probe_duration(path: &str) -> Result<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
            path,
        ])
        .output()
        .context("failed to run ffprobe — is ffmpeg installed? (brew install ffmpeg)")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffprobe failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let duration_str = stdout.trim();
    duration_str
        .parse::<f64>()
        .with_context(|| format!("failed to parse ffprobe duration output: '{duration_str}'"))
}

/// Trim a video from the end, keeping the last `keep_secs` seconds.
/// Uses re-encode to ensure clean keyframes (stream copy can produce corrupted mp4).
pub fn trim_from_end(input: &str, output: &str, keep_secs: f64) -> Result<()> {
    let total = probe_duration(input)?;
    let start = (total - keep_secs).max(0.0);

    let result = Command::new("ffmpeg")
        .args([
            "-y",
            "-ss",
            &format!("{start}"),
            "-i",
            input,
            "-t",
            &format!("{keep_secs}"),
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-crf",
            "23",
            "-c:a",
            "aac",
            output,
        ])
        .output()
        .context("failed to run ffmpeg — is ffmpeg installed? (brew install ffmpeg)")?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        anyhow::bail!("ffmpeg trim failed: {stderr}");
    }

    Ok(())
}

/// Prepare a source video for extend: probe duration, trim if too long, upload to TOS.
///
/// If the input is already an HTTP(S)/asset URL, it is returned as-is.
/// Local files are probed, trimmed if they exceed `max_duration` seconds,
/// then uploaded to TOS (S3-compatible) to get a public HTTPS URL.
///
/// Requires TOS_ACCESS_KEY, TOS_SECRET_KEY, TOS_BUCKET env vars for local files.
pub async fn prepare_source_video(input: &str, max_duration: u8) -> Result<String> {
    // URLs are passed through directly
    if input.starts_with("http://") || input.starts_with("https://") || input.starts_with("asset://") {
        return Ok(input.to_string());
    }

    let duration = probe_duration(input)?;

    if duration <= max_duration as f64 {
        // Upload as-is to TOS
        return tos::upload_file(input).await;
    }

    // Trim: keep the last max_duration seconds, then upload
    let trimmed_path = format!("{input}.seedance_trimmed.mp4");
    trim_from_end(input, &trimmed_path, max_duration as f64)?;

    let url = tos::upload_file(&trimmed_path).await;

    // Clean up temp file (best-effort)
    let _ = std::fs::remove_file(&trimmed_path);

    url
}

/// Upload a video file to TOS and return a public HTTPS URL for the API.
///
/// Unlike `prepare_source_video`, this does NOT trim the video — it uploads
/// the full file as-is. URLs are passed through directly.
pub async fn upload_video_for_api(input: &str) -> Result<String> {
    if input.starts_with("http://") || input.starts_with("https://") || input.starts_with("asset://") {
        return Ok(input.to_string());
    }
    tos::upload_file(input).await
}
