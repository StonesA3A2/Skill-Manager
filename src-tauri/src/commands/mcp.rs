use serde::Serialize;
use std::sync::Arc;
use tauri::State;

use crate::core::{
    error::AppError,
    mcp_adapters,
    skill_store::{McpServerRecord, McpServerTargetRecord, SkillStore},
};

#[derive(Debug, Serialize)]
pub struct McpServerDto {
    #[serde(flatten)]
    pub server: McpServerRecord,
    pub targets: Vec<McpServerTargetRecord>,
}

fn to_dto(store: &SkillStore, server: McpServerRecord) -> Result<McpServerDto, AppError> {
    let targets = store
        .get_targets_for_mcp_server(&server.id)
        .map_err(AppError::db)?;
    Ok(McpServerDto { server, targets })
}

#[tauri::command]
pub async fn list_mcp_servers(
    store: State<'_, Arc<SkillStore>>,
) -> Result<Vec<McpServerDto>, AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let servers = store.get_all_mcp_servers().map_err(AppError::db)?;
        servers
            .into_iter()
            .map(|s| to_dto(&store, s))
            .collect::<Result<Vec<_>, _>>()
    })
    .await?
}

#[tauri::command]
pub async fn create_mcp_server(
    name: String,
    description: Option<String>,
    command: String,
    args: Vec<String>,
    env: Option<std::collections::HashMap<String, String>>,
    store: State<'_, Arc<SkillStore>>,
) -> Result<McpServerDto, AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(AppError::invalid_input("Server name cannot be empty"));
        }
        if command.trim().is_empty() {
            return Err(AppError::invalid_input("Command cannot be empty"));
        }

        let now = chrono::Utc::now().timestamp_millis();
        let record = McpServerRecord {
            id: uuid::Uuid::new_v4().to_string(),
            name: trimmed_name.to_string(),
            description,
            command,
            args: serde_json::to_string(&args).map_err(AppError::io)?,
            env: env
                .map(|e| serde_json::to_string(&e))
                .transpose()
                .map_err(AppError::io)?,
            source_type: "manual".to_string(),
            source_skill_id: None,
            enabled: true,
            created_at: now,
            updated_at: now,
        };
        store.insert_mcp_server(&record).map_err(AppError::db)?;
        to_dto(&store, record)
    })
    .await?
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn update_mcp_server(
    id: String,
    name: String,
    description: Option<String>,
    command: String,
    args: Vec<String>,
    env: Option<std::collections::HashMap<String, String>>,
    enabled: bool,
    store: State<'_, Arc<SkillStore>>,
) -> Result<McpServerDto, AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut record = store
            .get_mcp_server_by_id(&id)
            .map_err(AppError::db)?
            .ok_or_else(|| AppError::not_found("MCP server not found"))?;

        record.name = name.trim().to_string();
        record.description = description;
        record.command = command;
        record.args = serde_json::to_string(&args).map_err(AppError::io)?;
        record.env = env
            .map(|e| serde_json::to_string(&e))
            .transpose()
            .map_err(AppError::io)?;
        record.enabled = enabled;
        record.updated_at = chrono::Utc::now().timestamp_millis();

        store.update_mcp_server(&record).map_err(AppError::db)?;

        // Re-deploy to every tool this server is already synced to, so an
        // edit (new args, renamed command, ...) doesn't silently leave the
        // stale version running in a tool's config until the next manual
        // deploy. Best-effort: one tool's write failure doesn't block the
        // others or the update itself — its target row just keeps reporting
        // the error via `last_error` for the UI to surface.
        let targets = store
            .get_targets_for_mcp_server(&record.id)
            .map_err(AppError::db)?;
        for target in targets {
            if let Some(adapter) = mcp_adapters::find_adapter(&target.tool) {
                redeploy_target(&store, &adapter, &record, &target.tool);
            }
        }

        to_dto(&store, record)
    })
    .await?
}

#[tauri::command]
pub async fn delete_mcp_server(
    id: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<(), AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let record = store
            .get_mcp_server_by_id(&id)
            .map_err(AppError::db)?
            .ok_or_else(|| AppError::not_found("MCP server not found"))?;

        // Best-effort undeploy from every tool before dropping the DB row —
        // a config file write failing here must not block deletion (the row
        // is going away regardless), so errors are swallowed rather than
        // surfaced. A leftover config entry after a failed write is no worse
        // than what deleting the row unconditionally would have left behind.
        let targets = store.get_targets_for_mcp_server(&id).map_err(AppError::db)?;
        for target in targets {
            if let Some(adapter) = mcp_adapters::find_adapter(&target.tool) {
                let _ = mcp_adapters::undeploy_server(&adapter, &record.name);
            }
        }

        store.delete_mcp_server(&id).map_err(AppError::db)?;
        Ok(())
    })
    .await?
}

/// Writes `record`'s current config into `tool` and records the resulting
/// (ok or error) status as that server's target row. Shared by
/// `deploy_mcp_server` and by `update_mcp_server`'s re-deploy-on-edit path.
fn redeploy_target(
    store: &SkillStore,
    adapter: &mcp_adapters::McpAdapter,
    record: &McpServerRecord,
    tool: &str,
) {
    let now = chrono::Utc::now().timestamp_millis();
    let (status, last_error) = match mcp_adapters::deploy_server(adapter, record) {
        Ok(()) => ("ok".to_string(), None),
        Err(e) => ("error".to_string(), Some(e.to_string())),
    };
    let _ = store.insert_mcp_server_target(&McpServerTargetRecord {
        id: uuid::Uuid::new_v4().to_string(),
        mcp_server_id: record.id.clone(),
        tool: tool.to_string(),
        status,
        synced_at: Some(now),
        last_error,
    });
}

#[tauri::command]
pub async fn deploy_mcp_server(
    id: String,
    tool: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<McpServerDto, AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let record = store
            .get_mcp_server_by_id(&id)
            .map_err(AppError::db)?
            .ok_or_else(|| AppError::not_found("MCP server not found"))?;
        let adapter = mcp_adapters::find_adapter(&tool)
            .ok_or_else(|| AppError::invalid_input(format!("Unknown tool: {tool}")))?;

        mcp_adapters::deploy_server(&adapter, &record).map_err(AppError::io)?;

        let now = chrono::Utc::now().timestamp_millis();
        store
            .insert_mcp_server_target(&McpServerTargetRecord {
                id: uuid::Uuid::new_v4().to_string(),
                mcp_server_id: record.id.clone(),
                tool: tool.clone(),
                status: "ok".to_string(),
                synced_at: Some(now),
                last_error: None,
            })
            .map_err(AppError::db)?;

        to_dto(&store, record)
    })
    .await?
}

#[tauri::command]
pub async fn undeploy_mcp_server(
    id: String,
    tool: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<McpServerDto, AppError> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let record = store
            .get_mcp_server_by_id(&id)
            .map_err(AppError::db)?
            .ok_or_else(|| AppError::not_found("MCP server not found"))?;
        let adapter = mcp_adapters::find_adapter(&tool)
            .ok_or_else(|| AppError::invalid_input(format!("Unknown tool: {tool}")))?;

        mcp_adapters::undeploy_server(&adapter, &record.name).map_err(AppError::io)?;
        store
            .delete_mcp_server_target(&record.id, &tool)
            .map_err(AppError::db)?;

        to_dto(&store, record)
    })
    .await?
}
