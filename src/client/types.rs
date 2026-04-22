use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CreateTaskRequest {
    pub model: String,
    pub content: Vec<ContentItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_last_frame: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentItem {
    Text {
        text: String,
    },
    ImageUrl {
        image_url: UrlRef,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
    },
    VideoUrl {
        video_url: UrlRef,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
    },
    AudioUrl {
        audio_url: UrlRef,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlRef {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskResponse {
    pub id: String,
    #[serde(default)]
    pub object: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub status: String,
    pub video_url: Option<String>,
    /// Some APIs return `url` instead of `video_url`
    pub url: Option<String>,
    pub content: Option<TaskContent>,
    pub error: Option<ApiError>,
    pub usage: Option<Usage>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl TaskResponse {
    pub fn resolved_video_url(&self) -> Option<&str> {
        self.video_url
            .as_deref()
            .or(self.url.as_deref())
            .or(self.content.as_ref().and_then(|c| c.video_url.as_deref()))
    }

    pub fn task_status(&self) -> TaskStatus {
        TaskStatus::from_str(&self.status)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskContent {
    pub video_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiError {
    pub message: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Submitted,
    Queued,
    Running,
    Succeeded,
    Failed,
    Expired,
    Cancelled,
    Unknown,
}

impl TaskStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "submitted" => Self::Submitted,
            "queued" => Self::Queued,
            "running" | "in_progress" => Self::Running,
            "succeeded" | "completed" => Self::Succeeded,
            "failed" => Self::Failed,
            "expired" => Self::Expired,
            "cancelled" => Self::Cancelled,
            _ => Self::Unknown,
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Expired | Self::Cancelled
        )
    }

    pub fn is_success(self) -> bool {
        self == Self::Succeeded
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Expired => "expired",
            Self::Cancelled => "cancelled",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}
