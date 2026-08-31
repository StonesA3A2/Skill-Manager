//! MCP server deployment: writing/removing entries in a target tool's MCP
//! config file.
//!
//! Unlike skills (one folder per skill, deployed by copy/symlink), an MCP
//! server is one entry inside a single JSON file that the target tool also
//! reads for its own purposes and that may contain servers the user (or
//! another app) configured directly. Every write here therefore:
//! 1. only ever touches the one key we own (`server.name` under the tool's
//!    top-level servers object) — never anything else in the file,
//! 2. backs up the file before writing,
//! 3. round-trips through `serde_json::Value` instead of a typed struct, so
//!    unknown top-level keys and unrelated server entries survive untouched.
//!
//! Update-plan item 2, step 1: Claude Code only. Further tools (Cursor,
//! Claude Desktop, ...) get their own `McpAdapter` once this is proven safe.

use anyhow::{bail, Context, Result};
use serde_json::{Map, Value};
use std::path::PathBuf;

use super::skill_store::McpServerRecord;

pub struct McpAdapter {
    pub key: &'static str,
    pub display_name: &'static str,
    /// Absolute path to the tool's MCP config file. A function (not a fixed
    /// path) so it's resolved lazily against the current `HOME`, matching
    /// the pattern `ToolAdapter` uses for skill directories.
    pub config_path: fn() -> Result<PathBuf>,
    /// The top-level JSON key under which this tool keeps its server map,
    /// e.g. `"mcpServers"`. Everything else in the file is left untouched.
    pub servers_key: &'static str,
}

pub fn claude_code_adapter() -> McpAdapter {
    McpAdapter {
        key: "claude_code",
        display_name: "Claude Code",
        config_path: || {
            dirs::home_dir()
                .map(|home| home.join(".claude.json"))
                .context("cannot determine home directory")
        },
        servers_key: "mcpServers",
    }
}

/// `~/.cursor/mcp.json`, key `mcpServers` — verified against Cursor's own
/// docs (cursor.com/docs/context/mcp): "Create ~/.cursor/mcp.json in your
/// home directory for tools available everywhere."
pub fn cursor_adapter() -> McpAdapter {
    McpAdapter {
        key: "cursor",
        display_name: "Cursor",
        config_path: || {
            dirs::home_dir()
                .map(|home| home.join(".cursor").join("mcp.json"))
                .context("cannot determine home directory")
        },
        servers_key: "mcpServers",
    }
}

/// `~/.codeium/windsurf/mcp_config.json`, key `mcpServers` — verified
/// against Windsurf's docs. Note the filename differs from Cursor/Claude
/// Code's `mcp.json`/`.claude.json` — this is deliberate, not a typo.
pub fn windsurf_adapter() -> McpAdapter {
    McpAdapter {
        key: "windsurf",
        display_name: "Windsurf",
        config_path: || {
            dirs::home_dir()
                .map(|home| home.join(".codeium").join("windsurf").join("mcp_config.json"))
                .context("cannot determine home directory")
        },
        servers_key: "mcpServers",
    }
}

/// Windows: `%APPDATA%\Claude\claude_desktop_config.json`. macOS:
/// `~/Library/Application Support/Claude/claude_desktop_config.json`. Both
/// verified against modelcontextprotocol.io's own quickstart. `dirs::config_dir()`
/// resolves to `%APPDATA%` on Windows and `~/Library/Application Support` on
/// macOS, so one path expression covers both — no separate Linux path is
/// documented upstream since Claude Desktop does not ship there.
pub fn claude_desktop_adapter() -> McpAdapter {
    McpAdapter {
        key: "claude_desktop",
        display_name: "Claude Desktop",
        config_path: || {
            dirs::config_dir()
                .map(|dir| dir.join("Claude").join("claude_desktop_config.json"))
                .context("cannot determine config directory")
        },
        servers_key: "mcpServers",
    }
}

pub fn all_adapters() -> Vec<McpAdapter> {
    vec![
        claude_code_adapter(),
        cursor_adapter(),
        windsurf_adapter(),
        claude_desktop_adapter(),
    ]
}

pub fn find_adapter(tool: &str) -> Option<McpAdapter> {
    all_adapters().into_iter().find(|a| a.key == tool)
}

/// Build the JSON entry for a server from its DB record. `args`/`env` are
/// stored as JSON-encoded strings in the DB; malformed JSON here would mean
/// the record was corrupted some other way, so this fails loudly rather than
/// silently deploying an empty array/object.
fn server_entry(server: &McpServerRecord) -> Result<Value> {
    let args: Value = serde_json::from_str(&server.args)
        .with_context(|| format!("mcp_servers.args for '{}' is not valid JSON", server.name))?;
    let mut entry = Map::new();
    entry.insert("command".to_string(), Value::String(server.command.clone()));
    entry.insert("args".to_string(), args);
    if let Some(env) = &server.env {
        let env_value: Value = serde_json::from_str(env)
            .with_context(|| format!("mcp_servers.env for '{}' is not valid JSON", server.name))?;
        entry.insert("env".to_string(), env_value);
    }
    Ok(Value::Object(entry))
}

use super::json_config_io::{read_config, write_config};

/// Writes/updates `server`'s entry in the adapter's config file. Only the
/// key `server.name` under `servers_key` is touched — every other key in the
/// file, including other servers, is preserved byte-for-byte apart from
/// JSON re-serialization.
pub fn deploy_server(adapter: &McpAdapter, server: &McpServerRecord) -> Result<()> {
    let path = (adapter.config_path)()?;
    let mut config = read_config(&path)?;
    let servers = config
        .entry(adapter.servers_key.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let Value::Object(servers_map) = servers else {
        bail!(
            "{:?}: '{}' is not a JSON object, refusing to overwrite",
            path,
            adapter.servers_key
        );
    };
    servers_map.insert(server.name.clone(), server_entry(server)?);
    write_config(&path, &config)
}

/// Removes `server_name`'s entry from the adapter's config file, if present.
/// A no-op (not an error) when the file, the servers key, or the entry
/// itself doesn't exist — undeploying something already gone is success.
pub fn undeploy_server(adapter: &McpAdapter, server_name: &str) -> Result<()> {
    let path = (adapter.config_path)()?;
    let mut config = read_config(&path)?;
    let Some(Value::Object(servers_map)) = config.get_mut(adapter.servers_key) else {
        return Ok(());
    };
    if servers_map.remove(server_name).is_none() {
        return Ok(());
    }
    write_config(&path, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn sample_server(name: &str) -> McpServerRecord {
        McpServerRecord {
            id: format!("id-{name}"),
            name: name.to_string(),
            description: None,
            command: "npx".to_string(),
            args: "[\"-y\", \"some-mcp-server\"]".to_string(),
            env: None,
            source_type: "manual".to_string(),
            source_skill_id: None,
            enabled: true,
            created_at: 0,
            updated_at: 0,
        }
    }

    // `McpAdapter::config_path` is a plain `fn() -> Result<PathBuf>` (not a
    // closure) so production adapters stay trivially `Copy`/`'static`. Tests
    // can't capture a per-test tempdir path in that function pointer, so
    // they instead stash it in a thread-local the pointer reads from.
    thread_local! {
        static TEST_PATH: std::cell::RefCell<PathBuf> = std::cell::RefCell::new(PathBuf::new());
    }

    fn adapter_at(path: PathBuf) -> McpAdapter {
        TEST_PATH.with(|p| *p.borrow_mut() = path);
        McpAdapter {
            key: "test_tool",
            display_name: "Test Tool",
            config_path: || Ok(TEST_PATH.with(|p| p.borrow().clone())),
            servers_key: "mcpServers",
        }
    }

    #[test]
    fn deploy_creates_file_with_servers_key() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("config.json");
        let adapter = adapter_at(config_path.clone());

        deploy_server(&adapter, &sample_server("my-server")).unwrap();

        let raw = fs::read_to_string(&config_path).unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed["mcpServers"]["my-server"]["command"], "npx");
    }

    #[test]
    fn deploy_preserves_unrelated_keys_and_other_servers() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("config.json");
        fs::write(
            &config_path,
            r#"{"someUnrelatedSetting": true, "mcpServers": {"other-server": {"command": "foo", "args": []}}}"#,
        )
        .unwrap();
        let adapter = adapter_at(config_path.clone());

        deploy_server(&adapter, &sample_server("my-server")).unwrap();

        let raw = fs::read_to_string(&config_path).unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed["someUnrelatedSetting"], true);
        assert_eq!(parsed["mcpServers"]["other-server"]["command"], "foo");
        assert_eq!(parsed["mcpServers"]["my-server"]["command"], "npx");
    }

    #[test]
    fn deploy_writes_backup_of_existing_file() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("config.json");
        fs::write(&config_path, r#"{"mcpServers": {}}"#).unwrap();
        let adapter = adapter_at(config_path.clone());

        deploy_server(&adapter, &sample_server("my-server")).unwrap();

        let has_backup = fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().starts_with("config.json.bak-"));
        assert!(has_backup, "expected a config.json.bak-* file after deploy");
    }

    #[test]
    fn undeploy_removes_only_named_server() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("config.json");
        fs::write(
            &config_path,
            r#"{"mcpServers": {"my-server": {"command": "npx", "args": []}, "other-server": {"command": "foo", "args": []}}}"#,
        )
        .unwrap();
        let adapter = adapter_at(config_path.clone());

        undeploy_server(&adapter, "my-server").unwrap();

        let raw = fs::read_to_string(&config_path).unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        assert!(parsed["mcpServers"].get("my-server").is_none());
        assert_eq!(parsed["mcpServers"]["other-server"]["command"], "foo");
    }

    #[test]
    fn undeploy_missing_file_is_a_no_op() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("does-not-exist.json");
        let adapter = adapter_at(config_path.clone());

        undeploy_server(&adapter, "my-server").unwrap();

        assert!(!config_path.exists());
    }

    #[test]
    fn all_adapters_have_distinct_keys_and_resolvable_paths() {
        let adapters = all_adapters();
        let mut keys: Vec<&str> = adapters.iter().map(|a| a.key).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), adapters.len(), "adapter keys must be unique");

        for adapter in &adapters {
            let path = (adapter.config_path)()
                .unwrap_or_else(|e| panic!("{}: config_path failed: {e}", adapter.key));
            assert!(
                path.is_absolute(),
                "{}: config_path must be absolute, got {:?}",
                adapter.key,
                path
            );
        }
    }

    #[test]
    fn cursor_adapter_targets_dot_cursor_mcp_json() {
        let path = (cursor_adapter().config_path)().unwrap();
        assert!(path.ends_with(".cursor/mcp.json") || path.ends_with(".cursor\\mcp.json"));
    }

    #[test]
    fn windsurf_adapter_targets_codeium_windsurf_mcp_config_json() {
        let path = (windsurf_adapter().config_path)().unwrap();
        let rel = path.strip_prefix(dirs::home_dir().unwrap()).unwrap();
        assert_eq!(rel, Path::new(".codeium").join("windsurf").join("mcp_config.json"));
    }

    #[test]
    fn claude_desktop_adapter_targets_claude_config_dir() {
        let path = (claude_desktop_adapter().config_path)().unwrap();
        let rel = path.strip_prefix(dirs::config_dir().unwrap()).unwrap();
        assert_eq!(rel, Path::new("Claude").join("claude_desktop_config.json"));
    }
}
