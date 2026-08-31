//! Claude Code plugin deployment: registering a plugin marketplace and
//! toggling a plugin on/off in a tool's own settings file.
//!
//! A "plugin" here is a Claude Code marketplace entry, verified against this
//! project's own live `~/.claude/settings.json` (it already uses this exact
//! shape for its `ecc` marketplace):
//! ```json
//! "extraKnownMarketplaces": {
//!   "ecc": { "source": { "source": "git", "url": "https://github.com/affaan-m/ECC.git" } }
//! },
//! "enabledPlugins": { "ecc@ecc": false }
//! ```
//! `enabledPlugins` keys are `<plugin_id>@<marketplace_key>`. Disabling a
//! plugin sets its value to `false` rather than removing the key — matching
//! how Claude Code's own UI behaves when you uncheck a plugin (the
//! marketplace registration and the plugin's presence in the list both
//! survive; only its enabled state flips).
//!
//! Unlike MCP server entries (each one fully own by whoever named it),
//! `extraKnownMarketplaces` is comparatively low-risk to overwrite: it holds
//! only `{key, source}`, and this module only ever writes an entry whose key
//! matches a marketplace already tracked in our own `plugin_marketplaces`
//! table, so we're never guessing at unrelated user-configured marketplaces.

use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::path::PathBuf;

use super::json_config_io::{read_config, write_config};

pub struct PluginAdapter {
    pub key: &'static str,
    pub display_name: &'static str,
    pub config_path: fn() -> Result<PathBuf>,
}

/// `~/.claude/settings.json` — Claude Code's own settings file, the same one
/// this app already edits for hooks (see `commands/settings.rs`).
pub fn claude_code_adapter() -> PluginAdapter {
    PluginAdapter {
        key: "claude_code",
        display_name: "Claude Code",
        config_path: || {
            dirs::home_dir()
                .map(|home| home.join(".claude").join("settings.json"))
                .context("cannot determine home directory")
        },
    }
}

pub fn all_adapters() -> Vec<PluginAdapter> {
    vec![claude_code_adapter()]
}

pub fn find_adapter(tool: &str) -> Option<PluginAdapter> {
    all_adapters().into_iter().find(|a| a.key == tool)
}

fn enabled_key(plugin_id: &str, marketplace_key: &str) -> String {
    format!("{plugin_id}@{marketplace_key}")
}

fn object_entry<'a>(config: &'a mut Map<String, Value>, key: &str) -> &'a mut Map<String, Value> {
    let needs_reset = !matches!(config.get(key), Some(Value::Object(_)));
    if needs_reset {
        config.insert(key.to_string(), Value::Object(Map::new()));
    }
    config.get_mut(key).unwrap().as_object_mut().unwrap()
}

/// Registers (or updates) the marketplace, then sets the plugin's
/// `enabledPlugins` entry to `true`.
pub fn deploy_plugin(
    adapter: &PluginAdapter,
    marketplace_key: &str,
    marketplace_url: &str,
    plugin_id: &str,
) -> Result<()> {
    let path = (adapter.config_path)()?;
    let mut config = read_config(&path)?;

    let mut source = Map::new();
    source.insert("source".to_string(), Value::String("git".to_string()));
    source.insert("url".to_string(), Value::String(marketplace_url.to_string()));
    let mut marketplace_entry = Map::new();
    marketplace_entry.insert("source".to_string(), Value::Object(source));
    object_entry(&mut config, "extraKnownMarketplaces")
        .insert(marketplace_key.to_string(), Value::Object(marketplace_entry));

    object_entry(&mut config, "enabledPlugins").insert(
        enabled_key(plugin_id, marketplace_key),
        Value::Bool(true),
    );

    write_config(&path, &config)
}

/// Sets the plugin's `enabledPlugins` entry to `false`. Leaves the
/// marketplace registration and the key itself in place — see module docs.
pub fn undeploy_plugin(adapter: &PluginAdapter, marketplace_key: &str, plugin_id: &str) -> Result<()> {
    let path = (adapter.config_path)()?;
    let mut config = read_config(&path)?;
    let Some(Value::Object(enabled)) = config.get_mut("enabledPlugins") else {
        return Ok(());
    };
    let key = enabled_key(plugin_id, marketplace_key);
    if !enabled.contains_key(&key) {
        return Ok(());
    }
    enabled.insert(key, Value::Bool(false));
    write_config(&path, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    thread_local! {
        static TEST_PATH: std::cell::RefCell<PathBuf> = std::cell::RefCell::new(PathBuf::new());
    }

    fn adapter_at(path: PathBuf) -> PluginAdapter {
        TEST_PATH.with(|p| *p.borrow_mut() = path);
        PluginAdapter {
            key: "test_tool",
            display_name: "Test Tool",
            config_path: || Ok(TEST_PATH.with(|p| p.borrow().clone())),
        }
    }

    #[test]
    fn deploy_registers_marketplace_and_enables_plugin() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("settings.json");
        let adapter = adapter_at(config_path.clone());

        deploy_plugin(&adapter, "ecc", "https://github.com/affaan-m/ECC.git", "ecc").unwrap();

        let raw = fs::read_to_string(&config_path).unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(
            parsed["extraKnownMarketplaces"]["ecc"]["source"]["url"],
            "https://github.com/affaan-m/ECC.git"
        );
        assert_eq!(parsed["enabledPlugins"]["ecc@ecc"], true);
    }

    #[test]
    fn deploy_preserves_unrelated_settings_keys() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("settings.json");
        fs::write(&config_path, r#"{"theme": "dark", "hooks": {"Stop": []}}"#).unwrap();
        let adapter = adapter_at(config_path.clone());

        deploy_plugin(&adapter, "ecc", "https://example.com/ecc.git", "ecc").unwrap();

        let raw = fs::read_to_string(&config_path).unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed["theme"], "dark");
        assert_eq!(parsed["hooks"]["Stop"], serde_json::json!([]));
    }

    #[test]
    fn undeploy_disables_without_removing_marketplace_or_key() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("settings.json");
        let adapter = adapter_at(config_path.clone());
        deploy_plugin(&adapter, "ecc", "https://example.com/ecc.git", "ecc").unwrap();

        undeploy_plugin(&adapter, "ecc", "ecc").unwrap();

        let raw = fs::read_to_string(&config_path).unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed["enabledPlugins"]["ecc@ecc"], false);
        assert!(parsed["extraKnownMarketplaces"].get("ecc").is_some());
    }

    #[test]
    fn undeploy_missing_file_is_a_no_op() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("does-not-exist.json");
        let adapter = adapter_at(config_path.clone());

        undeploy_plugin(&adapter, "ecc", "ecc").unwrap();

        assert!(!config_path.exists());
    }

    #[test]
    fn deploy_recovers_when_extra_marketplaces_key_is_not_an_object() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("settings.json");
        fs::write(&config_path, r#"{"extraKnownMarketplaces": "not-an-object"}"#).unwrap();
        let adapter = adapter_at(config_path.clone());

        deploy_plugin(&adapter, "ecc", "https://example.com/ecc.git", "ecc").unwrap();

        let raw = fs::read_to_string(&config_path).unwrap();
        let parsed: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(
            parsed["extraKnownMarketplaces"]["ecc"]["source"]["url"],
            "https://example.com/ecc.git"
        );
    }

    #[test]
    fn all_adapters_have_distinct_keys_and_resolvable_paths() {
        let adapters = all_adapters();
        let mut keys: Vec<&str> = adapters.iter().map(|a| a.key).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), adapters.len());
        for adapter in &adapters {
            let path = (adapter.config_path)().unwrap();
            assert!(path.is_absolute());
            assert!(path.ends_with(Path::new(".claude").join("settings.json")));
        }
    }
}
