//! Shared read/backup/write helpers for surgically editing one JSON object
//! inside a config file another application also reads — used by
//! `mcp_adapters` (each tool's MCP server list) and `plugin_adapters` (Claude
//! Code's `settings.json` marketplace/plugin entries). Kept generic over
//! `serde_json::Value` rather than a typed struct so unknown keys in the
//! target file always round-trip untouched.

use anyhow::{bail, Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub fn read_config(path: &Path) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw = fs::read_to_string(path).with_context(|| format!("reading {:?}", path))?;
    if raw.trim().is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_str::<Value>(&raw)? {
        Value::Object(map) => Ok(map),
        other => bail!("{:?} does not contain a JSON object at its root (found {})", path, other),
    }
}

fn backup_path(path: &Path) -> PathBuf {
    let suffix = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let mut backup = path.to_path_buf();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("config.json");
    backup.set_file_name(format!("{file_name}.bak-{suffix}"));
    backup
}

/// Writes `config` to `path`, creating parent directories and — when `path`
/// already exists — a timestamped backup copy first.
pub fn write_config(path: &Path, config: &Map<String, Value>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if path.exists() {
        fs::copy(path, backup_path(path)).with_context(|| format!("backing up {:?}", path))?;
    }
    let serialized = serde_json::to_string_pretty(config)?;
    fs::write(path, serialized).with_context(|| format!("writing {:?}", path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn read_missing_file_returns_empty_map() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("does-not-exist.json");
        assert!(read_config(&path).unwrap().is_empty());
    }

    #[test]
    fn write_then_read_round_trips() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("config.json");
        let mut config = Map::new();
        config.insert("hello".to_string(), Value::String("world".to_string()));
        write_config(&path, &config).unwrap();
        assert_eq!(read_config(&path).unwrap(), config);
    }

    #[test]
    fn write_backs_up_existing_file() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("config.json");
        fs::write(&path, r#"{"a": 1}"#).unwrap();
        write_config(&path, &Map::new()).unwrap();
        let has_backup = fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().starts_with("config.json.bak-"));
        assert!(has_backup);
    }
}
