use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

use crate::commands::skills::{collect_git_skill_dirs, resolve_skill_dir, GitSkillPreview};
use crate::core::{
    error::AppError,
    git_fetcher, plugin_adapters,
    skill_metadata,
    skill_store::{PluginMarketplaceRecord, PluginRecord, PluginTargetRecord, SkillStore},
};

#[derive(Debug, Serialize)]
pub struct PluginDto {
    pub id: String,
    pub plugin_id: String,
    pub name: Option<String>,
    pub enabled: bool,
    pub marketplace_key: String,
    pub marketplace_url: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub targets: Vec<PluginTargetRecord>,
}

fn to_dto(store: &SkillStore, plugin: PluginRecord, marketplace: &PluginMarketplaceRecord) -> Result<PluginDto, AppError> {
    let targets = store.get_targets_for_plugin(&plugin.id).map_err(AppError::db)?;
    Ok(PluginDto {
        id: plugin.id,
        plugin_id: plugin.plugin_id,
        name: plugin.name,
        enabled: plugin.enabled,
        marketplace_key: marketplace.key.clone(),
        marketplace_url: marketplace.source_url.clone(),
        created_at: plugin.created_at,
        updated_at: plugin.updated_at,
        targets,
    })
}

fn load_marketplace(store: &SkillStore, marketplace_id: &str) -> Result<PluginMarketplaceRecord, AppError> {
    store
        .get_all_plugin_marketplaces()
        .map_err(AppError::db)?
        .into_iter()
        .find(|m| m.id == marketplace_id)
        .ok_or_else(|| AppError::not_found("Plugin marketplace not found"))
}

#[tauri::command]
pub async fn list_plugins(store: State<'_, Arc<SkillStore>>) -> Result<Vec<PluginDto>, AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let plugins = store.get_all_plugins().map_err(AppError::db)?;
        plugins
            .into_iter()
            .map(|p| {
                let marketplace = load_marketplace(&store, &p.marketplace_id)?;
                to_dto(&store, p, &marketplace)
            })
            .collect::<Result<Vec<_>, _>>()
    })
    .await?
}

#[tauri::command]
pub async fn create_plugin(
    marketplace_key: String,
    marketplace_url: String,
    plugin_id: String,
    name: Option<String>,
    store: State<'_, Arc<SkillStore>>,
) -> Result<PluginDto, AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let marketplace_key = marketplace_key.trim();
        let marketplace_url = marketplace_url.trim();
        let plugin_id_trimmed = plugin_id.trim();
        if marketplace_key.is_empty() || marketplace_url.is_empty() || plugin_id_trimmed.is_empty() {
            return Err(AppError::invalid_input(
                "Marketplace key, marketplace URL, and plugin id are all required",
            ));
        }

        let now = chrono::Utc::now().timestamp_millis();
        let marketplace = match store
            .get_plugin_marketplace_by_key(marketplace_key)
            .map_err(AppError::db)?
        {
            Some(existing) => existing,
            None => {
                let record = PluginMarketplaceRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    key: marketplace_key.to_string(),
                    source_url: marketplace_url.to_string(),
                    created_at: now,
                    updated_at: now,
                };
                store.upsert_plugin_marketplace(&record).map_err(AppError::db)?;
                record
            }
        };

        if let Some(existing) =
            store.get_plugin_by_marketplace_and_plugin_id(&marketplace.id, plugin_id_trimmed).map_err(AppError::db)?
        {
            return Err(AppError::invalid_input(format!(
                "'{}' from marketplace '{}' is already tracked",
                existing.plugin_id, marketplace.key
            )));
        }

        let record = PluginRecord {
            id: uuid::Uuid::new_v4().to_string(),
            marketplace_id: marketplace.id.clone(),
            plugin_id: plugin_id_trimmed.to_string(),
            name,
            enabled: false,
            created_at: now,
            updated_at: now,
        };
        store.insert_plugin(&record).map_err(AppError::db)?;
        to_dto(&store, record, &marketplace)
    })
    .await?
}

#[tauri::command]
pub async fn delete_plugin(id: String, store: State<'_, Arc<SkillStore>>) -> Result<(), AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let plugin = store
            .get_plugin_by_id(&id)
            .map_err(AppError::db)?
            .ok_or_else(|| AppError::not_found("Plugin not found"))?;
        let marketplace = load_marketplace(&store, &plugin.marketplace_id)?;

        // Best-effort: disable everywhere it's deployed before dropping the
        // row, mirroring delete_mcp_server. A settings.json write failure
        // here must not block deletion.
        for target in store.get_targets_for_plugin(&id).map_err(AppError::db)? {
            if let Some(adapter) = plugin_adapters::find_adapter(&target.tool) {
                let _ = plugin_adapters::undeploy_plugin(&adapter, &marketplace.key, &plugin.plugin_id);
            }
        }

        store.delete_plugin(&id).map_err(AppError::db)?;
        Ok(())
    })
    .await?
}

#[tauri::command]
pub async fn deploy_plugin(
    id: String,
    tool: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<PluginDto, AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let plugin = store
            .get_plugin_by_id(&id)
            .map_err(AppError::db)?
            .ok_or_else(|| AppError::not_found("Plugin not found"))?;
        let marketplace = load_marketplace(&store, &plugin.marketplace_id)?;
        let adapter = plugin_adapters::find_adapter(&tool)
            .ok_or_else(|| AppError::invalid_input(format!("Unknown tool: {tool}")))?;

        plugin_adapters::deploy_plugin(&adapter, &marketplace.key, &marketplace.source_url, &plugin.plugin_id)
            .map_err(AppError::io)?;

        let now = chrono::Utc::now().timestamp_millis();
        store
            .insert_plugin_target(&PluginTargetRecord {
                id: uuid::Uuid::new_v4().to_string(),
                plugin_id: plugin.id.clone(),
                tool: tool.clone(),
                status: "ok".to_string(),
                synced_at: Some(now),
                last_error: None,
            })
            .map_err(AppError::db)?;
        store.update_plugin_enabled(&plugin.id, true, now).map_err(AppError::db)?;

        let updated = store
            .get_plugin_by_id(&plugin.id)
            .map_err(AppError::db)?
            .ok_or_else(|| AppError::not_found("Plugin not found"))?;
        to_dto(&store, updated, &marketplace)
    })
    .await?
}

#[tauri::command]
pub async fn undeploy_plugin(
    id: String,
    tool: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<PluginDto, AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let plugin = store
            .get_plugin_by_id(&id)
            .map_err(AppError::db)?
            .ok_or_else(|| AppError::not_found("Plugin not found"))?;
        let marketplace = load_marketplace(&store, &plugin.marketplace_id)?;
        let adapter = plugin_adapters::find_adapter(&tool)
            .ok_or_else(|| AppError::invalid_input(format!("Unknown tool: {tool}")))?;

        plugin_adapters::undeploy_plugin(&adapter, &marketplace.key, &plugin.plugin_id).map_err(AppError::io)?;
        store.delete_plugin_target(&plugin.id, &tool).map_err(AppError::db)?;

        let now = chrono::Utc::now().timestamp_millis();
        let remaining_targets = store.get_targets_for_plugin(&plugin.id).map_err(AppError::db)?;
        store
            .update_plugin_enabled(&plugin.id, !remaining_targets.is_empty(), now)
            .map_err(AppError::db)?;

        let updated = store
            .get_plugin_by_id(&plugin.id)
            .map_err(AppError::db)?
            .ok_or_else(|| AppError::not_found("Plugin not found"))?;
        to_dto(&store, updated, &marketplace)
    })
    .await?
}

#[derive(Debug, Deserialize)]
struct MarketplaceManifestPlugin {
    name: String,
    #[serde(default)]
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MarketplaceManifest {
    #[serde(default)]
    plugins: Vec<MarketplaceManifestPlugin>,
}

/// Lists the skills bundled inside `plugin`'s source, by cloning its
/// marketplace repo (or reusing the local clone cache — see
/// `git_fetcher::clone_repo_ref_with_progress`) and reading
/// `.claude-plugin/marketplace.json` to find which subpath this plugin's
/// entry points at (defaults to the repo root when the manifest omits
/// `source` or can't be found — most single-plugin marketplaces work this
/// way). Read-only: this never installs anything into the skill library,
/// it only answers "what's in here".
#[tauri::command]
pub async fn list_plugin_skills(
    id: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<Vec<GitSkillPreview>, AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let plugin = store
            .get_plugin_by_id(&id)
            .map_err(AppError::db)?
            .ok_or_else(|| AppError::not_found("Plugin not found"))?;
        let marketplace = load_marketplace(&store, &plugin.marketplace_id)?;
        let proxy_url = store.get_setting("proxy_url").ok().flatten();

        let parsed = git_fetcher::parse_git_source_resolved(&marketplace.source_url, proxy_url.as_deref());
        let repo_dir = git_fetcher::clone_repo_ref_with_progress(
            &parsed.clone_url,
            parsed.branch.as_deref(),
            None,
            proxy_url.as_deref(),
            None,
        )
        .map_err(AppError::classify_git_error)?;

        let manifest_path = repo_dir.join(".claude-plugin").join("marketplace.json");
        let plugin_source = fs_err_to_none(std::fs::read_to_string(&manifest_path))
            .and_then(|raw| serde_json::from_str::<MarketplaceManifest>(&raw).ok())
            .and_then(|manifest| {
                manifest
                    .plugins
                    .into_iter()
                    .find(|p| p.name == plugin.plugin_id)
                    .and_then(|p| p.source)
            });

        let skill_root = resolve_skill_dir(&repo_dir, plugin_source.as_deref(), None)?;
        let dirs = collect_git_skill_dirs(&skill_root);
        let skills: Vec<GitSkillPreview> = dirs
            .iter()
            .map(|dir| {
                let meta = skill_metadata::parse_skill_md(dir);
                let rel_path = dir
                    .strip_prefix(&skill_root)
                    .unwrap_or(dir)
                    .to_string_lossy()
                    .replace('\\', "/");
                let basename = dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| rel_path.clone());
                let name = meta.name.filter(|s| !s.trim().is_empty()).unwrap_or(basename);
                GitSkillPreview {
                    rel_path,
                    name,
                    description: meta.description,
                }
            })
            .collect();

        Ok(skills)
    })
    .await?
}

fn fs_err_to_none<T>(result: std::io::Result<T>) -> Option<T> {
    result.ok()
}
