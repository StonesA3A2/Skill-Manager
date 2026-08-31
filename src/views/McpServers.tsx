import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Plug, Plus, Trash2, Pencil, Loader2, RefreshCw, X, Check } from "lucide-react";
import * as api from "../lib/tauri";
import type { McpServer } from "../lib/tauri";
import { getErrorMessage } from "../lib/error";
import { ConfirmDialog } from "../components/ConfirmDialog";

// Must mirror `core::mcp_adapters::all_adapters()` exactly — listing a tool
// here without a matching backend adapter would let the user "deploy" to
// something that always fails.
const SUPPORTED_TOOLS = [
  { key: "claude_code", labelKey: "mcp.tool.claudeCode" },
  { key: "cursor", labelKey: "mcp.tool.cursor" },
  { key: "windsurf", labelKey: "mcp.tool.windsurf" },
  { key: "claude_desktop", labelKey: "mcp.tool.claudeDesktop" },
];

interface ServerFormState {
  name: string;
  command: string;
  argsText: string;
  envText: string;
  enabled: boolean;
}

const EMPTY_FORM: ServerFormState = { name: "", command: "", argsText: "", envText: "", enabled: true };

function parseArgs(text: string): string[] {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function parseEnv(text: string): Record<string, string> | null {
  const entries = text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const idx = line.indexOf("=");
      return idx === -1 ? null : ([line.slice(0, idx).trim(), line.slice(idx + 1).trim()] as const);
    })
    .filter((pair): pair is readonly [string, string] => pair !== null && pair[0].length > 0);
  return entries.length > 0 ? Object.fromEntries(entries) : null;
}

function formatArgs(argsJson: string): string {
  try {
    const parsed = JSON.parse(argsJson);
    return Array.isArray(parsed) ? parsed.join("\n") : "";
  } catch {
    return "";
  }
}

function formatEnv(envJson: string | null): string {
  if (!envJson) return "";
  try {
    const parsed = JSON.parse(envJson) as Record<string, string>;
    return Object.entries(parsed)
      .map(([k, v]) => `${k}=${v}`)
      .join("\n");
  } catch {
    return "";
  }
}

export function McpServers() {
  const { t } = useTranslation();
  const [servers, setServers] = useState<McpServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [formOpen, setFormOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [form, setForm] = useState<ServerFormState>(EMPTY_FORM);
  const [saving, setSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<McpServer | null>(null);
  const [busyKey, setBusyKey] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const list = await api.listMcpServers();
      setServers(list);
      setError(null);
    } catch (err) {
      setError(getErrorMessage(err, t("common.error")));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const openCreateForm = () => {
    setEditingId(null);
    setForm(EMPTY_FORM);
    setFormOpen(true);
  };

  const openEditForm = (server: McpServer) => {
    setEditingId(server.id);
    setForm({
      name: server.name,
      command: server.command,
      argsText: formatArgs(server.args),
      envText: formatEnv(server.env),
      enabled: server.enabled,
    });
    setFormOpen(true);
  };

  const handleSave = async () => {
    const name = form.name.trim();
    const command = form.command.trim();
    if (!name || !command) {
      toast.error(t("mcp.form.validationError"));
      return;
    }
    setSaving(true);
    try {
      const args = parseArgs(form.argsText);
      const env = parseEnv(form.envText);
      if (editingId) {
        await api.updateMcpServer(editingId, name, null, command, args, env, form.enabled);
      } else {
        await api.createMcpServer(name, null, command, args, env);
      }
      setFormOpen(false);
      await refresh();
      toast.success(editingId ? t("mcp.form.updated") : t("mcp.form.created"));
    } catch (err) {
      toast.error(getErrorMessage(err, t("common.error")));
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    await api.deleteMcpServer(deleteTarget.id);
    await refresh();
    toast.success(t("mcp.deleted"));
  };

  const handleToggleDeploy = async (server: McpServer, tool: string) => {
    const target = server.targets.find((tgt) => tgt.tool === tool);
    const key = `${server.id}:${tool}`;
    setBusyKey(key);
    try {
      if (target) {
        await api.undeployMcpServer(server.id, tool);
      } else {
        await api.deployMcpServer(server.id, tool);
      }
      await refresh();
    } catch (err) {
      toast.error(getErrorMessage(err, t("common.error")));
    } finally {
      setBusyKey(null);
    }
  };

  return (
    <div className="app-page">
      <div className="app-page-header pr-2 pb-1 flex items-center justify-between gap-3">
        <div>
          <h1 className="app-page-title">{t("mcp.title")}</h1>
          <p className="mt-1 text-[13px] text-muted">{t("mcp.subtitle")}</p>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={refresh}
            disabled={loading}
            className="inline-flex h-8 items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 text-[13px] font-medium text-tertiary transition-colors hover:bg-surface-hover disabled:opacity-50"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
            {t("settings.refresh")}
          </button>
          <button type="button" onClick={openCreateForm} className="app-button-primary">
            <Plus className="h-4 w-4" />
            {t("mcp.addServer")}
          </button>
        </div>
      </div>

      {error && (
        <div className="app-panel border border-red-500/30 bg-red-500/10 p-4 text-[13px] text-red-400">
          {error}
        </div>
      )}

      {!loading && servers.length === 0 && !error ? (
        <div className="app-panel flex flex-col items-center gap-3 p-10 text-center">
          <Plug className="h-8 w-8 text-faint" />
          <p className="text-[13px] text-muted">{t("mcp.empty")}</p>
        </div>
      ) : (
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          {servers.map((server) => (
            <div key={server.id} className="flex flex-wrap items-center gap-3 px-4 py-3.5">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border bg-surface-hover">
                <Plug className="h-4 w-4 text-accent" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate text-[13px] font-medium text-secondary">{server.name}</p>
                <p className="truncate text-[12px] text-muted">
                  {server.command} {formatArgs(server.args).split("\n").join(" ")}
                </p>
              </div>
              <div className="flex flex-wrap items-center gap-1.5">
                {SUPPORTED_TOOLS.map((tool) => {
                  const target = server.targets.find((tgt) => tgt.tool === tool.key);
                  const deployed = !!target;
                  const key = `${server.id}:${tool.key}`;
                  return (
                    <button
                      key={tool.key}
                      type="button"
                      onClick={() => handleToggleDeploy(server, tool.key)}
                      disabled={busyKey === key}
                      title={target?.last_error ?? undefined}
                      className={`inline-flex items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-[12px] font-medium transition-colors disabled:opacity-50 ${
                        deployed
                          ? target.status === "error"
                            ? "border-red-500/40 bg-red-500/10 text-red-400 hover:bg-red-500/20"
                            : "border-accent-border bg-accent-bg text-accent-light hover:opacity-90"
                          : "border-border bg-surface-hover text-tertiary hover:bg-surface-active"
                      }`}
                    >
                      {busyKey === key ? (
                        <Loader2 className="h-3 w-3 animate-spin" />
                      ) : deployed ? (
                        <Check className="h-3 w-3" />
                      ) : null}
                      {t(tool.labelKey)}
                    </button>
                  );
                })}
              </div>
              <div className="flex items-center gap-1.5">
                <button
                  type="button"
                  onClick={() => openEditForm(server)}
                  className="rounded-lg border border-border bg-surface-hover p-1.5 text-tertiary transition-colors hover:bg-surface-active"
                  title={t("mcp.edit")}
                >
                  <Pencil className="h-3.5 w-3.5" />
                </button>
                <button
                  type="button"
                  onClick={() => setDeleteTarget(server)}
                  className="rounded-lg border border-border bg-surface-hover p-1.5 text-red-400 transition-colors hover:bg-red-500/10"
                  title={t("mcp.remove")}
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {formOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div className="absolute inset-0 bg-black/70 backdrop-blur-sm" onClick={() => setFormOpen(false)} />
          <div className="relative w-full max-w-md rounded-xl border border-border bg-surface p-5 shadow-2xl">
            <div className="mb-4 flex items-center justify-between">
              <h2 className="text-[14px] font-semibold text-primary">
                {editingId ? t("mcp.form.editTitle") : t("mcp.form.addTitle")}
              </h2>
              <button
                type="button"
                onClick={() => setFormOpen(false)}
                className="rounded p-1 text-muted transition-colors hover:text-secondary"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="space-y-3">
              <div>
                <label className="mb-1 block text-[13px] font-medium text-tertiary">{t("mcp.form.name")}</label>
                <input
                  type="text"
                  value={form.name}
                  onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                  placeholder={t("mcp.form.namePlaceholder")}
                  className="app-input w-full bg-background"
                />
              </div>
              <div>
                <label className="mb-1 block text-[13px] font-medium text-tertiary">{t("mcp.form.command")}</label>
                <input
                  type="text"
                  value={form.command}
                  onChange={(e) => setForm((f) => ({ ...f, command: e.target.value }))}
                  placeholder={t("mcp.form.commandPlaceholder")}
                  className="app-input w-full bg-background"
                />
              </div>
              <div>
                <label className="mb-1 block text-[13px] font-medium text-tertiary">{t("mcp.form.args")}</label>
                <textarea
                  value={form.argsText}
                  onChange={(e) => setForm((f) => ({ ...f, argsText: e.target.value }))}
                  placeholder={t("mcp.form.argsPlaceholder")}
                  rows={3}
                  className="app-input w-full resize-none bg-background font-mono text-[12px]"
                />
              </div>
              <div>
                <label className="mb-1 block text-[13px] font-medium text-tertiary">{t("mcp.form.env")}</label>
                <textarea
                  value={form.envText}
                  onChange={(e) => setForm((f) => ({ ...f, envText: e.target.value }))}
                  placeholder={t("mcp.form.envPlaceholder")}
                  rows={2}
                  className="app-input w-full resize-none bg-background font-mono text-[12px]"
                />
              </div>
              {editingId && (
                <label className="flex items-center gap-2 text-[13px] text-tertiary">
                  <input
                    type="checkbox"
                    checked={form.enabled}
                    onChange={(e) => setForm((f) => ({ ...f, enabled: e.target.checked }))}
                    className="h-4 w-4 accent-accent"
                  />
                  {t("mcp.form.enabled")}
                </label>
              )}
            </div>

            <div className="mt-5 flex justify-end gap-2">
              <button
                type="button"
                onClick={() => setFormOpen(false)}
                className="px-3 py-1.5 rounded-lg text-[13px] font-medium text-tertiary hover:text-secondary hover:bg-surface-hover transition-colors"
              >
                {t("common.cancel")}
              </button>
              <button type="button" onClick={handleSave} disabled={saving} className="app-button-primary">
                {saving ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
                {t("common.save")}
              </button>
            </div>
          </div>
        </div>
      )}

      <ConfirmDialog
        open={!!deleteTarget}
        tone="danger"
        title={t("mcp.remove")}
        message={t("mcp.deleteConfirm", { name: deleteTarget?.name ?? "" })}
        onClose={() => setDeleteTarget(null)}
        onConfirm={handleDelete}
      />
    </div>
  );
}
