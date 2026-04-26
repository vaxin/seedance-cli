// ═══════════════════════════════════════════════════════
// 1. client::types — TaskStatus, ContentItem, TaskResponse
// ═══════════════════════════════════════════════════════

mod types_tests {
    use seedance_cli::client::types::*;

    #[test]
    fn task_status_from_str_standard_values() {
        assert_eq!(TaskStatus::from_str("submitted"), TaskStatus::Submitted);
        assert_eq!(TaskStatus::from_str("queued"), TaskStatus::Queued);
        assert_eq!(TaskStatus::from_str("running"), TaskStatus::Running);
        assert_eq!(TaskStatus::from_str("succeeded"), TaskStatus::Succeeded);
        assert_eq!(TaskStatus::from_str("failed"), TaskStatus::Failed);
        assert_eq!(TaskStatus::from_str("expired"), TaskStatus::Expired);
        assert_eq!(TaskStatus::from_str("cancelled"), TaskStatus::Cancelled);
    }

    #[test]
    fn task_status_from_str_aliases() {
        assert_eq!(TaskStatus::from_str("in_progress"), TaskStatus::Running);
        assert_eq!(TaskStatus::from_str("completed"), TaskStatus::Succeeded);
    }

    #[test]
    fn task_status_case_insensitive() {
        assert_eq!(TaskStatus::from_str("SUCCEEDED"), TaskStatus::Succeeded);
        assert_eq!(TaskStatus::from_str("Running"), TaskStatus::Running);
        assert_eq!(TaskStatus::from_str("FAILED"), TaskStatus::Failed);
    }

    #[test]
    fn task_status_unknown_value() {
        assert_eq!(TaskStatus::from_str("bogus"), TaskStatus::Unknown);
        assert_eq!(TaskStatus::from_str(""), TaskStatus::Unknown);
    }

    #[test]
    fn task_status_is_terminal() {
        assert!(TaskStatus::Succeeded.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Expired.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());
        assert!(!TaskStatus::Submitted.is_terminal());
        assert!(!TaskStatus::Queued.is_terminal());
        assert!(!TaskStatus::Running.is_terminal());
        assert!(!TaskStatus::Unknown.is_terminal());
    }

    #[test]
    fn task_status_is_success() {
        assert!(TaskStatus::Succeeded.is_success());
        assert!(!TaskStatus::Failed.is_success());
        assert!(!TaskStatus::Running.is_success());
    }

    #[test]
    fn task_status_display() {
        assert_eq!(format!("{}", TaskStatus::Succeeded), "succeeded");
        assert_eq!(format!("{}", TaskStatus::Running), "running");
    }

    #[test]
    fn content_item_text_serializes_correctly() {
        let item = ContentItem::Text {
            text: "hello world".into(),
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "hello world");
    }

    #[test]
    fn content_item_image_url_serializes_with_role() {
        let item = ContentItem::ImageUrl {
            image_url: UrlRef {
                url: "https://example.com/img.jpg".into(),
            },
            role: Some("reference_image".into()),
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["type"], "image_url");
        assert_eq!(json["image_url"]["url"], "https://example.com/img.jpg");
        assert_eq!(json["role"], "reference_image");
    }

    #[test]
    fn content_item_image_url_omits_none_role() {
        let item = ContentItem::ImageUrl {
            image_url: UrlRef {
                url: "https://example.com/img.jpg".into(),
            },
            role: None,
        };
        let json = serde_json::to_value(&item).unwrap();
        assert!(json.get("role").is_none());
    }

    #[test]
    fn content_item_video_url_serializes() {
        let item = ContentItem::VideoUrl {
            video_url: UrlRef {
                url: "https://example.com/clip.mp4".into(),
            },
            role: Some("reference_video".into()),
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["type"], "video_url");
        assert_eq!(json["video_url"]["url"], "https://example.com/clip.mp4");
    }

    #[test]
    fn content_item_audio_url_serializes() {
        let item = ContentItem::AudioUrl {
            audio_url: UrlRef {
                url: "https://example.com/music.mp3".into(),
            },
            role: None,
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["type"], "audio_url");
    }

    #[test]
    fn create_task_request_omits_none_fields() {
        let req = CreateTaskRequest {
            model: "test-model".into(),
            content: vec![ContentItem::Text {
                text: "test".into(),
            }],
            resolution: Some("1080p".into()),
            ratio: Some("16:9".into()),
            duration: Some(5),
            watermark: Some(false),
            generate_audio: None,
            seed: None,
            return_last_frame: None,
            callback_url: None,
            tools: None,
            service_tier: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(json.get("generate_audio").is_none());
        assert!(json.get("seed").is_none());
        assert!(json.get("return_last_frame").is_none());
        assert!(json.get("callback_url").is_none());
        assert!(json.get("tools").is_none());
        assert!(json.get("service_tier").is_none());
        assert!(json.get("prompt").is_none());
        assert_eq!(json["watermark"], false);
    }

    #[test]
    fn create_task_request_with_tools() {
        let req = CreateTaskRequest {
            model: "test-model".into(),
            content: vec![ContentItem::Text {
                text: "test".into(),
            }],
            resolution: None,
            ratio: None,
            duration: None,
            watermark: None,
            generate_audio: None,
            seed: None,
            return_last_frame: None,
            callback_url: None,
            tools: Some(vec![Tool {
                tool_type: "web_search".into(),
            }]),
            service_tier: Some("flex".into()),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["tools"][0]["type"], "web_search");
        assert_eq!(json["service_tier"], "flex");
    }

    #[test]
    fn content_item_image_url_with_first_frame_role() {
        let item = ContentItem::ImageUrl {
            image_url: UrlRef {
                url: "https://example.com/frame.png".into(),
            },
            role: Some("first_frame".into()),
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["role"], "first_frame");
    }

    #[test]
    fn content_item_image_url_with_last_frame_role() {
        let item = ContentItem::ImageUrl {
            image_url: UrlRef {
                url: "https://example.com/frame.png".into(),
            },
            role: Some("last_frame".into()),
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["role"], "last_frame");
    }

    #[test]
    fn task_response_deserialize_with_last_frame_image_url() {
        let json = r#"{
            "id": "cgt-lf",
            "status": "succeeded",
            "content": {
                "video_url": "https://example.com/v.mp4",
                "last_frame_image_url": "https://example.com/last_frame.png"
            }
        }"#;
        let resp: TaskResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            resp.last_frame_image_url(),
            Some("https://example.com/last_frame.png")
        );
        assert_eq!(
            resp.resolved_video_url(),
            Some("https://example.com/v.mp4")
        );
    }

    #[test]
    fn task_response_deserialize_with_tool_usage() {
        let json = r#"{
            "id": "cgt-ws",
            "status": "succeeded",
            "video_url": "https://example.com/v.mp4",
            "usage": {
                "completion_tokens": 50000,
                "total_tokens": 50000,
                "tool_usage": {"web_search": 2}
            }
        }"#;
        let resp: TaskResponse = serde_json::from_str(json).unwrap();
        let usage = resp.usage.as_ref().unwrap();
        assert_eq!(usage.tool_usage.as_ref().unwrap().web_search, Some(2));
    }

    #[test]
    fn task_response_deserialize_with_video_url() {
        let json = r#"{
            "id": "cgt-123",
            "status": "succeeded",
            "video_url": "https://example.com/video.mp4"
        }"#;
        let resp: TaskResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "cgt-123");
        assert_eq!(resp.resolved_video_url(), Some("https://example.com/video.mp4"));
        assert!(resp.task_status().is_success());
    }

    #[test]
    fn task_response_deserialize_with_url_fallback() {
        let json = r#"{
            "id": "cgt-456",
            "status": "completed",
            "url": "https://example.com/alt-video.mp4"
        }"#;
        let resp: TaskResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            resp.resolved_video_url(),
            Some("https://example.com/alt-video.mp4")
        );
        assert!(resp.task_status().is_success());
    }

    #[test]
    fn task_response_video_url_prefers_video_url_over_url() {
        let json = r#"{
            "id": "cgt-789",
            "status": "succeeded",
            "video_url": "https://primary.com/v.mp4",
            "url": "https://fallback.com/v.mp4"
        }"#;
        let resp: TaskResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            resp.resolved_video_url(),
            Some("https://primary.com/v.mp4")
        );
    }

    #[test]
    fn task_response_deserialize_failed_with_error() {
        let json = r#"{
            "id": "cgt-err",
            "status": "failed",
            "error": {"message": "content blocked", "code": "SAFETY_FILTER"}
        }"#;
        let resp: TaskResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.task_status(), TaskStatus::Failed);
        assert_eq!(
            resp.error.as_ref().unwrap().message.as_deref(),
            Some("content blocked")
        );
    }

    #[test]
    fn task_response_deserialize_with_usage() {
        let json = r#"{
            "id": "cgt-usage",
            "status": "succeeded",
            "video_url": "https://example.com/v.mp4",
            "usage": {"completion_tokens": 102960, "total_tokens": 102960}
        }"#;
        let resp: TaskResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.usage.as_ref().unwrap().total_tokens, Some(102960));
    }
}

// ═══════════════════════════════════════════════════════
// 2. config — defaults, get/set, TOML round-trip
// ═══════════════════════════════════════════════════════

mod config_tests {
    use seedance_cli::config::AppConfig;

    #[test]
    fn defaults_are_correct() {
        let cfg = AppConfig::with_defaults();
        assert_eq!(cfg.base_url, "https://ark.cn-beijing.volces.com/api/v3");
        assert_eq!(cfg.model, "standard");
        assert_eq!(cfg.resolution, "1080p");
        assert_eq!(cfg.ratio, "16:9");
        assert_eq!(cfg.duration, 5);
        assert_eq!(cfg.output_dir, ".");
        assert!(cfg.api_key.is_none());
    }

    #[test]
    fn get_known_keys() {
        let cfg = AppConfig::with_defaults();
        assert_eq!(cfg.get("model"), Some("standard".into()));
        assert_eq!(cfg.get("resolution"), Some("1080p".into()));
        assert_eq!(cfg.get("ratio"), Some("16:9".into()));
        assert_eq!(cfg.get("duration"), Some("5".into()));
        assert_eq!(cfg.get("output_dir"), Some(".".into()));
        assert_eq!(cfg.get("api_key"), None);
    }

    #[test]
    fn get_unknown_key_returns_none() {
        let cfg = AppConfig::with_defaults();
        assert_eq!(cfg.get("nonexistent"), None);
    }

    #[test]
    fn set_known_keys() {
        let mut cfg = AppConfig::with_defaults();
        cfg.set("model", "fast").unwrap();
        assert_eq!(cfg.model, "fast");

        cfg.set("duration", "10").unwrap();
        assert_eq!(cfg.duration, 10);

        cfg.set("api_key", "sk-test123").unwrap();
        assert_eq!(cfg.api_key, Some("sk-test123".into()));
    }

    #[test]
    fn set_unknown_key_fails() {
        let mut cfg = AppConfig::with_defaults();
        assert!(cfg.set("bogus_key", "value").is_err());
    }

    #[test]
    fn set_duration_invalid_number_fails() {
        let mut cfg = AppConfig::with_defaults();
        assert!(cfg.set("duration", "not-a-number").is_err());
    }

    #[test]
    fn toml_round_trip() {
        let mut cfg = AppConfig::with_defaults();
        cfg.api_key = Some("sk-roundtrip".into());
        cfg.model = "fast".into();
        cfg.duration = 12;

        let serialized = toml::to_string_pretty(&cfg).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.api_key, Some("sk-roundtrip".into()));
        assert_eq!(deserialized.model, "fast");
        assert_eq!(deserialized.duration, 12);
        assert_eq!(deserialized.resolution, "1080p");
    }

    #[test]
    fn toml_partial_deserialize_uses_defaults() {
        let partial = r#"
            api_key = "sk-partial"
            model = "fast"
        "#;
        let cfg: AppConfig = toml::from_str(partial).unwrap();
        assert_eq!(cfg.api_key, Some("sk-partial".into()));
        assert_eq!(cfg.model, "fast");
        assert_eq!(cfg.resolution, "1080p");
        assert_eq!(cfg.duration, 5);
    }

    #[test]
    fn resolve_api_key_from_config() {
        let mut cfg = AppConfig::with_defaults();
        cfg.api_key = Some("sk-from-config".into());
        // Clear env var to avoid interference
        std::env::remove_var("ARK_API_KEY");
        assert_eq!(cfg.resolve_api_key().unwrap(), "sk-from-config");
    }

    #[test]
    fn resolve_api_key_env_takes_precedence() {
        let mut cfg = AppConfig::with_defaults();
        cfg.api_key = Some("sk-from-config".into());
        std::env::set_var("ARK_API_KEY", "sk-from-env");
        assert_eq!(cfg.resolve_api_key().unwrap(), "sk-from-env");
        std::env::remove_var("ARK_API_KEY");
    }

    #[test]
    fn resolve_api_key_fails_when_not_set() {
        let cfg = AppConfig::with_defaults();
        std::env::remove_var("ARK_API_KEY");
        assert!(cfg.resolve_api_key().is_err());
    }
}

// ═══════════════════════════════════════════════════════
// 3. core::upload — URL detection, MIME guessing, base64
// ═══════════════════════════════════════════════════════

mod upload_tests {
    use seedance_cli::core::upload;

    #[test]
    fn http_url_passes_through() {
        let url = "https://example.com/image.jpg";
        assert_eq!(upload::resolve_file_ref(url).unwrap(), url);
    }

    #[test]
    fn http_insecure_url_passes_through() {
        let url = "http://example.com/video.mp4";
        assert_eq!(upload::resolve_file_ref(url).unwrap(), url);
    }

    #[test]
    fn asset_url_passes_through() {
        let url = "asset://asset-20260401123823-6d4x2";
        assert_eq!(upload::resolve_file_ref(url).unwrap(), url);
    }

    #[test]
    fn nonexistent_local_file_fails() {
        let result = upload::resolve_file_ref("/tmp/seedance_test_nonexistent_xyz.png");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("file not found"));
    }

    #[test]
    fn local_file_returns_base64_data_uri() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jpg");
        std::fs::write(&path, b"fake-jpg-data").unwrap();

        let result = upload::resolve_file_ref(path.to_str().unwrap()).unwrap();
        assert!(result.starts_with("data:image/jpeg;base64,"));
    }

    #[test]
    fn local_png_file_correct_mime() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.png");
        std::fs::write(&path, b"fake-png-data").unwrap();

        let result = upload::resolve_file_ref(path.to_str().unwrap()).unwrap();
        assert!(result.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn local_mp4_file_correct_mime() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clip.mp4");
        std::fs::write(&path, b"fake-mp4-data").unwrap();

        let result = upload::resolve_file_ref(path.to_str().unwrap()).unwrap();
        assert!(result.starts_with("data:video/mp4;base64,"));
    }

    #[test]
    fn local_mp3_file_correct_mime() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audio.mp3");
        std::fs::write(&path, b"fake-mp3-data").unwrap();

        let result = upload::resolve_file_ref(path.to_str().unwrap()).unwrap();
        assert!(result.starts_with("data:audio/mpeg;base64,"));
    }

    #[test]
    fn guess_mime_all_types() {
        assert_eq!(upload::guess_mime("photo.jpg"), "image/jpeg");
        assert_eq!(upload::guess_mime("photo.jpeg"), "image/jpeg");
        assert_eq!(upload::guess_mime("photo.JPEG"), "image/jpeg");
        assert_eq!(upload::guess_mime("photo.png"), "image/png");
        assert_eq!(upload::guess_mime("photo.webp"), "image/webp");
        assert_eq!(upload::guess_mime("video.mp4"), "video/mp4");
        assert_eq!(upload::guess_mime("video.mov"), "video/quicktime");
        assert_eq!(upload::guess_mime("audio.mp3"), "audio/mpeg");
        assert_eq!(upload::guess_mime("file.xyz"), "application/octet-stream");
    }
}

// ═══════════════════════════════════════════════════════
// 4. store — TaskStore CRUD
// ═══════════════════════════════════════════════════════

mod store_tests {
    use seedance_cli::store::TaskStore;

    #[test]
    fn insert_and_list() {
        let store = TaskStore::open_in_memory().unwrap();
        store.insert("cgt-001", "a golden retriever", "standard").unwrap();
        store.insert("cgt-002", "a cat astronaut", "fast").unwrap();

        let records = store.list(10, None).unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn update_status() {
        let store = TaskStore::open_in_memory().unwrap();
        store.insert("cgt-003", "test prompt", "standard").unwrap();
        store
            .update_status("cgt-003", "succeeded", Some("https://example.com/v.mp4"))
            .unwrap();

        let records = store.list(10, None).unwrap();
        assert_eq!(records[0].status, "succeeded");
        assert_eq!(
            records[0].video_url.as_deref(),
            Some("https://example.com/v.mp4")
        );
    }

    #[test]
    fn update_output_path() {
        let store = TaskStore::open_in_memory().unwrap();
        store.insert("cgt-004", "test prompt", "standard").unwrap();
        store.update_output_path("cgt-004", "/tmp/output.mp4").unwrap();

        let records = store.list(10, None).unwrap();
        assert_eq!(records[0].output_path.as_deref(), Some("/tmp/output.mp4"));
    }

    #[test]
    fn list_with_status_filter() {
        let store = TaskStore::open_in_memory().unwrap();
        store.insert("cgt-010", "prompt a", "standard").unwrap();
        store.insert("cgt-011", "prompt b", "standard").unwrap();
        store.update_status("cgt-010", "succeeded", None).unwrap();
        store.update_status("cgt-011", "failed", None).unwrap();

        let succeeded = store.list(10, Some("succeeded")).unwrap();
        assert_eq!(succeeded.len(), 1);
        assert_eq!(succeeded[0].task_id, "cgt-010");

        let failed = store.list(10, Some("failed")).unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].task_id, "cgt-011");
    }

    #[test]
    fn list_with_all_filter_returns_everything() {
        let store = TaskStore::open_in_memory().unwrap();
        store.insert("cgt-020", "prompt a", "standard").unwrap();
        store.insert("cgt-021", "prompt b", "standard").unwrap();
        store.update_status("cgt-020", "succeeded", None).unwrap();

        let all = store.list(10, Some("all")).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn list_respects_limit() {
        let store = TaskStore::open_in_memory().unwrap();
        for i in 0..10 {
            store
                .insert(&format!("cgt-{i:03}"), &format!("prompt {i}"), "standard")
                .unwrap();
        }

        let limited = store.list(3, None).unwrap();
        assert_eq!(limited.len(), 3);
    }

    #[test]
    fn insert_or_replace_updates_existing() {
        let store = TaskStore::open_in_memory().unwrap();
        store.insert("cgt-030", "original prompt", "standard").unwrap();
        store.insert("cgt-030", "updated prompt", "fast").unwrap();

        let records = store.list(10, None).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].prompt, "updated prompt");
        assert_eq!(records[0].model, "fast");
    }
}

// ═══════════════════════════════════════════════════════
// 5. cli::generate — validation, prompt resolution, model ID
// ═══════════════════════════════════════════════════════

mod generate_tests {
    use seedance_cli::cli::generate::{resolve_model_id, resolve_prompt, validate_inputs, GenerateArgs};
    use std::io::Write;

    fn default_args() -> GenerateArgs {
        GenerateArgs {
            prompt: "test".into(),
            model: "standard".into(),
            duration: 5,
            ratio: "16:9".into(),
            resolution: "1080p".into(),
            seed: None,
            watermark: false,
            audio_gen: false,
            return_last_frame: false,
            callback: None,
            web_search: false,
            service_tier: None,
            image: vec![],
            video: vec![],
            audio: vec![],
            first_frame: None,
            last_frame: None,
            wait: false,
            output: None,
            timeout: 300,
            poll_interval: 10,
            strict: false,
            quiet: false,
            json: false,
        }
    }

    #[test]
    fn resolve_model_id_standard() {
        assert_eq!(
            resolve_model_id("standard"),
            "doubao-seedance-2-0-260128"
        );
    }

    #[test]
    fn resolve_model_id_std_alias() {
        assert_eq!(resolve_model_id("std"), "doubao-seedance-2-0-260128");
    }

    #[test]
    fn resolve_model_id_fast() {
        assert_eq!(
            resolve_model_id("fast"),
            "doubao-seedance-2-0-fast-260128"
        );
    }

    #[test]
    fn resolve_model_id_raw_passthrough() {
        assert_eq!(
            resolve_model_id("custom-model-v3"),
            "custom-model-v3"
        );
    }

    #[test]
    fn resolve_prompt_plain_text() {
        assert_eq!(resolve_prompt("hello world").unwrap(), "hello world");
    }

    #[test]
    fn resolve_prompt_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("prompt.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "  a golden retriever running  ").unwrap();

        let result = resolve_prompt(&format!("@{}", path.display())).unwrap();
        assert_eq!(result, "a golden retriever running");
    }

    #[test]
    fn resolve_prompt_from_nonexistent_file_fails() {
        assert!(resolve_prompt("@/tmp/seedance_nonexistent_xyz.txt").is_err());
    }

    #[test]
    fn validate_duration_too_short() {
        let mut args = default_args();
        args.duration = 3;
        assert!(validate_inputs(&args).is_err());
    }

    #[test]
    fn validate_duration_too_long() {
        let mut args = default_args();
        args.duration = 16;
        assert!(validate_inputs(&args).is_err());
    }

    #[test]
    fn validate_duration_boundary_values() {
        let mut args = default_args();
        args.duration = 4;
        assert!(validate_inputs(&args).is_ok());
        args.duration = 15;
        assert!(validate_inputs(&args).is_ok());
    }

    #[test]
    fn validate_too_many_images() {
        let mut args = default_args();
        args.image = (0..10).map(|i| format!("https://img/{i}.jpg")).collect();
        assert!(validate_inputs(&args).is_err());
    }

    #[test]
    fn validate_max_images_ok() {
        let mut args = default_args();
        args.image = (0..9).map(|i| format!("https://img/{i}.jpg")).collect();
        assert!(validate_inputs(&args).is_ok());
    }

    #[test]
    fn validate_too_many_videos() {
        let mut args = default_args();
        args.video = (0..4).map(|i| format!("https://vid/{i}.mp4")).collect();
        assert!(validate_inputs(&args).is_err());
    }

    #[test]
    fn validate_too_many_audio() {
        let mut args = default_args();
        args.audio = (0..4).map(|i| format!("https://aud/{i}.mp3")).collect();
        assert!(validate_inputs(&args).is_err());
    }

    #[test]
    fn validate_rule_of_12_total() {
        let mut args = default_args();
        args.image = (0..9).map(|i| format!("https://img/{i}.jpg")).collect();
        args.video = (0..3).map(|i| format!("https://vid/{i}.mp4")).collect();
        // 9 + 3 = 12, OK
        assert!(validate_inputs(&args).is_ok());

        // Add first_frame = 13 total, should fail
        args.first_frame = Some("https://ff.jpg".into());
        assert!(validate_inputs(&args).is_err());
    }

    #[test]
    fn validate_first_last_frame_count_in_total() {
        let mut args = default_args();
        args.image = (0..9).map(|i| format!("https://img/{i}.jpg")).collect();
        args.first_frame = Some("https://first.jpg".into());
        args.last_frame = Some("https://last.jpg".into());
        // 9 + 1 + 1 = 11, OK
        assert!(validate_inputs(&args).is_ok());

        args.video = vec!["https://v.mp4".into(), "https://v2.mp4".into()];
        // 9 + 2 + 1 + 1 = 13, FAIL
        assert!(validate_inputs(&args).is_err());
    }
}

// ═══════════════════════════════════════════════════════
// 6. cli::extend — Args parsing
// ═══════════════════════════════════════════════════════

mod extend_tests {
    use clap::Parser;
    use seedance_cli::cli::extend::ExtendArgs;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        args: ExtendArgs,
    }

    #[test]
    fn parse_single_source() {
        let cli = TestCli::try_parse_from([
            "test", "cgt-abc123", "向后延长视频1，角色走出房间",
        ])
        .unwrap();
        assert_eq!(cli.args.source, vec!["cgt-abc123"]);
        assert_eq!(cli.args.prompt, "向后延长视频1，角色走出房间");
        assert_eq!(cli.args.duration, 5);
        assert_eq!(cli.args.model, "standard");
    }

    #[test]
    fn parse_multiple_sources_for_bridging() {
        let cli = TestCli::try_parse_from([
            "test", "cgt-001", "cgt-002", "cgt-003",
            "视频1打开门，接视频2，镜头推进，接视频3",
        ])
        .unwrap();
        assert_eq!(cli.args.source, vec!["cgt-001", "cgt-002", "cgt-003"]);
    }

    #[test]
    fn parse_with_options() {
        let cli = TestCli::try_parse_from([
            "test", "cgt-abc", "延长内容",
            "--model", "fast",
            "--duration", "10",
            "--ratio", "9:16",
            "--audio-gen",
            "--return-last-frame",
            "--wait",
        ])
        .unwrap();
        assert_eq!(cli.args.model, "fast");
        assert_eq!(cli.args.duration, 10);
        assert_eq!(cli.args.ratio, "9:16");
        assert!(cli.args.audio_gen);
        assert!(cli.args.return_last_frame);
        assert!(cli.args.wait);
    }

    #[test]
    fn parse_with_image_refs() {
        let cli = TestCli::try_parse_from([
            "test", "cgt-abc", "延长并加入角色",
            "--image", "character.png",
            "--image", "asset://asset-123",
        ])
        .unwrap();
        assert_eq!(cli.args.image.len(), 2);
        assert_eq!(cli.args.image[1], "asset://asset-123");
    }

    #[test]
    fn reject_no_source() {
        let result = TestCli::try_parse_from(["test"]);
        assert!(result.is_err());
    }

    #[test]
    fn reject_too_many_sources() {
        let result = TestCli::try_parse_from([
            "test", "a", "b", "c", "d", "prompt",
        ]);
        assert!(result.is_err());
    }
}

// ═══════════════════════════════════════════════════════
// 7. cli::edit — Args parsing
// ═══════════════════════════════════════════════════════

mod edit_tests {
    use clap::Parser;
    use seedance_cli::cli::edit::EditArgs;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        args: EditArgs,
    }

    #[test]
    fn parse_basic_edit() {
        let cli = TestCli::try_parse_from([
            "test", "cgt-abc123", "将香水替换成面霜",
        ])
        .unwrap();
        assert_eq!(cli.args.source, "cgt-abc123");
        assert_eq!(cli.args.prompt, "将香水替换成面霜");
    }

    #[test]
    fn parse_edit_with_image() {
        let cli = TestCli::try_parse_from([
            "test", "cgt-abc", "替换成图片1中的面霜",
            "--image", "cream.jpg",
            "--duration", "8",
            "--audio-gen",
            "--wait",
        ])
        .unwrap();
        assert_eq!(cli.args.image, vec!["cream.jpg"]);
        assert_eq!(cli.args.duration, 8);
        assert!(cli.args.audio_gen);
        assert!(cli.args.wait);
    }

    #[test]
    fn parse_edit_with_audio() {
        let cli = TestCli::try_parse_from([
            "test", "cgt-abc", "替换配音",
            "--audio", "new_voice.mp3",
        ])
        .unwrap();
        assert_eq!(cli.args.audio, vec!["new_voice.mp3"]);
    }

    #[test]
    fn reject_missing_source() {
        let result = TestCli::try_parse_from(["test"]);
        assert!(result.is_err());
    }

    #[test]
    fn reject_missing_prompt() {
        let result = TestCli::try_parse_from(["test", "cgt-abc"]);
        assert!(result.is_err());
    }
}

// ═══════════════════════════════════════════════════════
// 8. poller — PollOptions defaults
// ═══════════════════════════════════════════════════════

mod poller_tests {
    use seedance_cli::core::poller::PollOptions;
    use std::time::Duration;

    #[test]
    fn default_poll_options() {
        let opts = PollOptions::default();
        assert_eq!(opts.timeout, Duration::from_secs(300));
        assert_eq!(opts.initial_interval, Duration::from_secs(10));
        assert!(!opts.strict);
    }
}
