use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::env;
use std::path::Path;

type HmacSha256 = Hmac<Sha256>;

fn hmac_sign(key: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key size");
    mac.update(msg);
    mac.finalize().into_bytes().to_vec()
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn get_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("environment variable {name} is not set"))
}

/// Upload a file to TOS (S3-compatible) and return the public HTTPS URL.
///
/// Required env vars:
///   TOS_ACCESS_KEY — Access Key ID
///   TOS_SECRET_KEY — Secret Access Key
///   TOS_BUCKET     — Bucket name
/// Optional:
///   TOS_ENDPOINT   — default: tos-cn-beijing.volces.com
///   TOS_REGION     — default: cn-beijing
pub async fn upload_file(file_path: &str) -> Result<String> {
    let access_key = get_env("TOS_ACCESS_KEY")?;
    let secret_key = get_env("TOS_SECRET_KEY")?;
    let bucket = get_env("TOS_BUCKET")?;
    let region = env::var("TOS_REGION").unwrap_or_else(|_| "cn-beijing".into());
    let endpoint = env::var("TOS_ENDPOINT").unwrap_or_else(|_| "tos-s3-cn-beijing.volces.com".into());

    // Read file
    let data = tokio::fs::read(file_path)
        .await
        .with_context(|| format!("failed to read {file_path}"))?;

    let filename = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4");

    let content_type = if filename.ends_with(".mp4") {
        "video/mp4"
    } else if filename.ends_with(".mov") {
        "video/quicktime"
    } else {
        "application/octet-stream"
    };

    // Object key: seedance-cli/{timestamp}_{uuid}_{filename}
    let now = chrono::Utc::now();
    let ts = now.format("%Y%m%dT%H%M%S").to_string();
    let id = uuid::Uuid::new_v4().to_string().split('-').next().unwrap().to_string();
    let key = format!("seedance-cli/{ts}_{id}_{filename}");

    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();
    let content_sha256 = sha256_hex(&data);
    let service = "s3";

    // Virtual-hosted style: {bucket}.{endpoint}
    let host = format!("{bucket}.{endpoint}");
    let canonical_uri = format!("/{key}");
    let canonical_querystring = "";

    // 8-hour auto-expiry
    let expires_at = (now + chrono::Duration::hours(8))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let canonical_headers = format!(
        "content-type:{content_type}\nhost:{host}\nx-amz-content-sha256:{content_sha256}\nx-amz-date:{amz_date}\nx-amz-meta-expire-at:{expires_at}\n"
    );
    let signed_headers = "content-type;host;x-amz-content-sha256;x-amz-date;x-amz-meta-expire-at";

    let canonical_request = format!(
        "PUT\n{canonical_uri}\n{canonical_querystring}\n{canonical_headers}\n{signed_headers}\n{content_sha256}"
    );

    // String to Sign
    let credential_scope = format!("{date_stamp}/{region}/{service}/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        sha256_hex(canonical_request.as_bytes())
    );

    // Signing key
    let date_key = hmac_sign(format!("AWS4{secret_key}").as_bytes(), date_stamp.as_bytes());
    let region_key = hmac_sign(&date_key, region.as_bytes());
    let service_key = hmac_sign(&region_key, service.as_bytes());
    let signing_key = hmac_sign(&service_key, b"aws4_request");

    let signature = hex::encode(hmac_sign(&signing_key, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={access_key}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}"
    );

    // ── Send PUT request ──

    let url = format!("https://{host}{canonical_uri}");
    let host_header = host.clone();

    let client = reqwest::Client::new();
    let resp = client
        .put(&url)
        .header("Host", &host_header)
        .header("x-amz-date", &amz_date)
        .header("x-amz-content-sha256", &content_sha256)
        .header("Content-Type", content_type)
        .header("x-amz-meta-expire-at", &expires_at)
        .header("Authorization", &authorization)
        .body(data)
        .send()
        .await
        .context("failed to upload to TOS")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("TOS upload failed (HTTP {status}): {body}");
    }

    // Public URL: https://{bucket}.{endpoint}/{key}
    let public_url = format!("https://{host_header}/{key}");
    Ok(public_url)
}

/// Delete a file from TOS by its public URL.
///
/// Parses the key from the URL, sends a SigV4-signed DELETE request.
pub async fn delete_file(public_url: &str) -> Result<()> {
    let access_key = get_env("TOS_ACCESS_KEY")?;
    let secret_key = get_env("TOS_SECRET_KEY")?;
    let bucket = get_env("TOS_BUCKET")?;
    let region = env::var("TOS_REGION").unwrap_or_else(|_| "cn-beijing".into());
    let endpoint = env::var("TOS_ENDPOINT").unwrap_or_else(|_| "tos-s3-cn-beijing.volces.com".into());

    let host = format!("{bucket}.{endpoint}");
    let prefix = format!("https://{host}/");
    let key = public_url
        .strip_prefix(&prefix)
        .with_context(|| format!("cannot extract TOS key from URL: {public_url}"))?;

    let now = chrono::Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();
    let payload_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    let canonical_uri = format!("/{key}");
    let canonical_headers = format!(
        "host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n"
    );
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";
    let canonical_request = format!(
        "DELETE\n{canonical_uri}\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
    );

    let service = "s3";
    let credential_scope = format!("{date_stamp}/{region}/{service}/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        sha256_hex(canonical_request.as_bytes())
    );

    let date_key = hmac_sign(format!("AWS4{secret_key}").as_bytes(), date_stamp.as_bytes());
    let region_key = hmac_sign(&date_key, region.as_bytes());
    let service_key = hmac_sign(&region_key, service.as_bytes());
    let signing_key = hmac_sign(&service_key, b"aws4_request");
    let signature = hex::encode(hmac_sign(&signing_key, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={access_key}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}"
    );

    let url = format!("https://{host}{canonical_uri}");
    let client = reqwest::Client::new();
    let resp = client
        .delete(&url)
        .header("Host", &host)
        .header("x-amz-date", &amz_date)
        .header("x-amz-content-sha256", payload_hash)
        .header("Authorization", &authorization)
        .send()
        .await
        .context("failed to delete from TOS")?;

    let status = resp.status();
    if status != 204 && status != 404 {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("TOS delete failed (HTTP {status}): {body}");
    }
    Ok(())
}

/// Clean up all expired objects under `seedance-cli/` prefix.
///
/// Object keys contain a timestamp: `seedance-cli/{timestamp}_{uuid}_{filename}`.
/// Objects older than 8 hours are deleted.
pub async fn cleanup_expired() -> Result<usize> {
    let access_key = get_env("TOS_ACCESS_KEY")?;
    let secret_key = get_env("TOS_SECRET_KEY")?;
    let bucket = get_env("TOS_BUCKET")?;
    let region = env::var("TOS_REGION").unwrap_or_else(|_| "cn-beijing".into());
    let endpoint = env::var("TOS_ENDPOINT").unwrap_or_else(|_| "tos-s3-cn-beijing.volces.com".into());
    let service = "s3";
    let host = format!("{bucket}.{endpoint}");
    let prefix = "seedance-cli/";

    let now = chrono::Utc::now();
    let cutoff = now - chrono::Duration::hours(8);
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();
    let payload_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    // List objects
    let canonical_uri = "/";
    let canonical_querystring = format!("prefix={prefix}");
    let canonical_headers = format!(
        "host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n"
    );
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";
    let canonical_request = format!(
        "GET\n{canonical_uri}\n{canonical_querystring}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
    );

    let credential_scope = format!("{date_stamp}/{region}/{service}/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        sha256_hex(canonical_request.as_bytes())
    );

    let date_key = hmac_sign(format!("AWS4{secret_key}").as_bytes(), date_stamp.as_bytes());
    let region_key = hmac_sign(&date_key, region.as_bytes());
    let service_key = hmac_sign(&region_key, service.as_bytes());
    let signing_key = hmac_sign(&service_key, b"aws4_request");
    let signature = hex::encode(hmac_sign(&signing_key, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={access_key}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}"
    );

    let list_url = format!("https://{host}/?prefix={prefix}");
    let client = reqwest::Client::new();
    let resp = client
        .get(&list_url)
        .header("Host", &host)
        .header("x-amz-date", &amz_date)
        .header("x-amz-content-sha256", payload_hash)
        .header("Authorization", &authorization)
        .send()
        .await
        .context("failed to list TOS objects")?;

    let body = resp.text().await.context("failed to read TOS list response")?;

    // Parse XML keys and extract timestamps
    let mut deleted = 0;
    let mut in_key = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "<Key>" {
            in_key = true;
        } else if trimmed == "</Key>" {
            in_key = false;
        } else if in_key {
            let key = trimmed.to_string();
            // Key format: seedance-cli/{timestamp}_{...}
            if let Some(rest) = key.strip_prefix("seedance-cli/") {
                // timestamp is 15 chars: YYYYMMDDTHHMMSS
                if rest.len() >= 15 {
                    let ts_str = &rest[..15];
                    let key_dt = chrono::NaiveDateTime::parse_from_str(ts_str, "%Y%m%dT%H%M%S")
                        .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc));
                    if let Ok(dt) = key_dt {
                        if dt < cutoff {
                            if delete_file(&format!("https://{host}/{key}")).await.is_ok() {
                                deleted += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(deleted)
}
