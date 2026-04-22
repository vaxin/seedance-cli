pub mod error;
pub mod types;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use types::{CreateTaskRequest, TaskResponse};

pub struct ArkClient {
    http: reqwest::Client,
    base_url: String,
}

impl ArkClient {
    pub fn new(base_url: &str, api_key: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))
                .context("invalid API key characters")?,
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    pub async fn create_task(&self, req: &CreateTaskRequest) -> Result<TaskResponse> {
        let url = format!(
            "{}/contents/generations/tasks",
            self.base_url
        );

        let resp = self.http.post(&url).json(req).send().await?;
        let status = resp.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            anyhow::bail!(error::ArkError::RateLimited);
        }

        let body = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!(error::ArkError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        let task: TaskResponse =
            serde_json::from_str(&body).with_context(|| format!("failed to parse response: {body}"))?;
        Ok(task)
    }

    pub async fn query_task(&self, task_id: &str) -> Result<TaskResponse> {
        let url = format!(
            "{}/contents/generations/tasks/{task_id}",
            self.base_url
        );

        let resp = self.http.get(&url).send().await?;
        let status = resp.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            anyhow::bail!(error::ArkError::RateLimited);
        }

        let body = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!(error::ArkError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        let task: TaskResponse =
            serde_json::from_str(&body).with_context(|| format!("failed to parse response: {body}"))?;
        Ok(task)
    }
}
