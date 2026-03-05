use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

pub fn write_failure_report(stage: &str, details: &Value) -> Option<PathBuf> {
    let dir = debug_dir();
    if fs::create_dir_all(&dir).is_err() {
        return None;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let stage_name = sanitize(stage);

    let filename = format!(
        "{}-{:03}-{}.json",
        now.as_secs(),
        now.subsec_millis(),
        stage_name
    );
    let path = dir.join(filename);

    let payload = serde_json::json!({
        "timestamp_unix_ms": now.as_millis(),
        "stage": stage,
        "details": details,
    });

    let body = serde_json::to_string_pretty(&payload).ok()?;
    if fs::write(&path, body).is_err() {
        return None;
    }

    let latest_path = dir.join("latest.json");
    let _ = fs::copy(&path, latest_path);

    Some(path)
}

fn debug_dir() -> PathBuf {
    if let Some(home_dir) = std::env::var_os("HOME") {
        return PathBuf::from(home_dir).join(".sweetgreen").join("debug");
    }

    PathBuf::from(".sweetgreen").join("debug")
}

fn sanitize(input: &str) -> String {
    let mut out = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();

    if out.is_empty() {
        out = "failure".to_string();
    }

    if out.len() > 64 {
        out.truncate(64);
    }

    out
}
