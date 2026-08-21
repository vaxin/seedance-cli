use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CreateTaskRequest {
    pub model: String,
    pub content: Vec<ContentItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratio: Option<String>,
    /// Duration in seconds; `-1` lets the model pick (Seedance 2.5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    /// Task type for Seedance 2.5+: "generate" / "edit" / "extend"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    /// Output container: "mp4" (default) or "mov" (Seedance 2.5+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
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

    pub fn last_frame_image_url(&self) -> Option<&str> {
        self.content
            .as_ref()
            .and_then(|c| c.last_frame_image_url.as_deref())
    }

    pub fn task_status(&self) -> TaskStatus {
        TaskStatus::from_str(&self.status)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskContent {
    pub video_url: Option<String>,
    pub url: Option<String>,
    pub last_frame_image_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiError {
    pub message: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolUsage {
    pub web_search: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub tool_usage: Option<ToolUsage>,
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

// ─────────────────────────────────────────────────────────────────
// Image generation (Seedream) — official Ark /images/generations API
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ImageGenRequest {
    pub model: String,
    pub prompt: String,
    /// Reference images for i2i / editing / multi-image fusion
    /// (URLs or base64 data URIs; max 10)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub image: Vec<String>,
    /// e.g. "2048x2048", "1K", "2K", "4K", "adaptive"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// "url" (default) or "b64_json"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Group/sequential image generation (Seedream 4.0+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequential_image_generation: Option<SequentialImageGeneration>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SequentialImageGeneration {
    /// Number of images in the group (2-4)
    pub max_images: u8,
    /// Let the model write its own variation prompt per image (default true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_default_prompt: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageGenResponse {
    pub model: Option<String>,
    pub created: Option<u64>,
    pub data: Vec<ImageGenItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageGenItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,
}
