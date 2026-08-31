import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Blocks, Plus, Trash2, Loader2, RefreshCw, X, Check, ChevronRight, ChevronDown } from "lucide-react";
import * as api from "../lib/tauri";
import type { Plugin, PluginSkillPreview } from "../lib/tauri";
import { getErrorMessage } from "../lib/error";
import { ConfirmDialog } from "../components/ConfirmDialog";

// Must mirror `core::plugin_adapters::all_adapters()` — only Claude Code has
// a backend adapter so far (it's the only tool whose plugin/marketplace
// format is documented and verified against this project's own settings.json).
const SUPPORTED_TOOLS = [{ key: "claude_code", labelKey: "mcp.tool.claudeCode" }];

interface FormState {
  marketplaceKey: string;
  marketplaceUrl: string;
  pluginId: string;
  name: string;
}

const EMPTY_FORM: FormState = { marketplaceKey: "", marketplaceUrl: "", pluginId: "", name: "" };

export function Plugins() {
  const { t } = useTranslation();
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [formOpen, setFormOpen] = useState(false);
  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [saving, setSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Plugin | null>(null);
  const [busyKey, setBusyKey] = useState<string | null>(null);
  const [expandedPluginId, setExpandedPluginId] = useState<string | null>(null);
  const [pluginSkills, setPluginSkills] = useState<Record<string, PluginSkillPreview[]>>({});
  const [loadingSkillsFor, setLoadingSkillsFor] = useState<string | null>(null);
  const [skillsError, setSkillsError] = useState<Record<string, string>>({});

  const toggleExpanded = useCallback(
    async (plugin: Plugin) => {
      if (expandedPluginId === plugin.id) {
        setExpandedPluginId(null);
        return;
      }
      setExpandedPluginId(plugin.id);
      if (pluginSkills[plugin.id]) return;
      setLoadingSkillsFor(plugin.id);
      try {
        const skills = await api.listPluginSkills(plugin.id);
        setPluginSkills((prev) => ({ ...prev, [plugin.id]: skills }));
        setSkillsError((prev) => {
          const next = { ...prev };
          delete next[plugin.id];
          return next;
        });
      } catch (err) {
        setSkillsError((prev) => ({ ...prev, [plugin.id]: getErrorMessage(err, t("common.error")) }));
      } finally {
        setLoadingSkillsFor(null);
      }
    },
    [expandedPluginId, pluginSkills, t]
  );

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setPlugins(await api.listPlugins());
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
    setForm(EMPTY_FORM);
    setFormOpen(true);
  };

  const handleSave = async () => {
    const marketplaceKey = form.marketplaceKey.trim();
    const marketplaceUrl = form.marketplaceUrl.trim();
    const pluginId = form.pluginId.trim();
    if (!marketplaceKey || !marketplaceUrl || !pluginId) {
      toast.error(t("plugins.form.validationError"));
      return;
    }
    setSaving(true);
    try {
      await api.createPlugin(marketplaceKey, marketplaceUrl, pluginId, form.name.trim() || null);
      setFormOpen(false);
      await refresh();
      toast.success(t("plugins.form.created"));
    } catch (err) {
      toast.error(getErrorMessage(err, t("common.error")));
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    await api.deletePlugin(deleteTarget.id);
    await refresh();
    toast.success(t("plugins.deleted"));
  };

  const handleToggleDeploy = async (plugin: Plugin, tool: string) => {
    const target = plugin.targets.find((tgt) => tgt.tool === tool);
    const key = `${plugin.id}:${tool}`;
    setBusyKey(key);
    try {
      if (target) {
        await api.undeployPlugin(plugin.id, tool);
      } else {
        await api.deployPlugin(plugin.id, tool);
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
          <h1 className="app-page-title">{t("plugins.title")}</h1>
          <p className="mt-1 text-[13px] text-muted">{t("plugins.subtitle")}</p>
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
            {t("plugins.addPlugin")}
          </button>
        </div>
      </div>

      {error && (
        <div className="app-panel border border-red-500/30 bg-red-500/10 p-4 text-[13px] text-red-400">
          {error}
        </div>
      )}

      {!loading && plugins.length === 0 && !error ? (
        <div className="app-panel flex flex-col items-center gap-3 p-10 text-center">
          <Blocks className="h-8 w-8 text-faint" />
          <p className="text-[13px] text-muted">{t("plugins.empty")}</p>
        </div>
      ) : (
        <div className="app-panel overflow-hidden divide-y divide-border-subtle">
          {plugins.map((plugin) => {
            const expanded = expandedPluginId === plugin.id;
            const skills = pluginSkills[plugin.id];
            return (
            <div key={plugin.id}>
            <div className="flex flex-wrap items-center gap-3 px-4 py-3.5">
              <button
                type="button"
                onClick={() => toggleExpanded(plugin)}
                className="shrink-0 text-faint transition-colors hover:text-secondary"
                title={t("plugins.showSkills")}
              >
                {expanded ? <ChevronDown className="h-3.5 w-3.5" /> : <ChevronRight className="h-3.5 w-3.5" />}
              </button>
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border bg-surface-hover">
                <Blocks className="h-4 w-4 text-accent" />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <p className="truncate text-[13px] font-medium text-secondary">
                    {plugin.name || plugin.plugin_id}
                  </p>
                  <span
                    className={`shrink-0 rounded-full border px-1.5 py-0.5 text-[10px] font-medium ${
                      plugin.enabled
                        ? "border-accent-border bg-accent-bg text-accent-light"
                        : "border-border-subtle bg-surface-hover text-faint"
                    }`}
                  >
                    {plugin.enabled ? t("plugins.active") : t("plugins.inactive")}
                  </span>
                </div>
                <p className="truncate text-[12px] text-muted">
                  {plugin.plugin_id}@{plugin.marketplace_key} — {plugin.marketplace_url}
                </p>
              </div>
              <div className="flex flex-wrap items-center gap-1.5">
                {SUPPORTED_TOOLS.map((tool) => {
                  const target = plugin.targets.find((tgt) => tgt.tool === tool.key);
                  const deployed = !!target;
                  const key = `${plugin.id}:${tool.key}`;
                  return (
                    <button
                      key={tool.key}
                      type="button"
                      onClick={() => handleToggleDeploy(plugin, tool.key)}
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
              <button
                type="button"
                onClick={() => setDeleteTarget(plugin)}
                className="rounded-lg border border-border bg-surface-hover p-1.5 text-red-400 transition-colors hover:bg-red-500/10"
                title={t("plugins.remove")}
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </div>
            {expanded && (
              <div className="border-t border-border-subtle bg-surface px-4 py-3 pl-12">
                {loadingSkillsFor === plugin.id ? (
                  <div className="flex items-center gap-2 text-[13px] text-muted">
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                    {t("plugins.loadingSkills")}
                  </div>
                ) : skillsError[plugin.id] ? (
                  <p className="text-[13px] text-red-400">{skillsError[plugin.id]}</p>
                ) : skills && skills.length > 0 ? (
                  <ul className="space-y-1.5">
                    {skills.map((skill) => (
                      <li key={skill.rel_path} className="text-[13px]">
                        <span className="font-medium text-secondary">{skill.name}</span>
                        {skill.description && (
                          <span className="text-muted"> — {skill.description}</span>
                        )}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="text-[13px] text-muted">{t("plugins.noSkillsFound")}</p>
                )}
              </div>
            )}
            </div>
            );
          })}
        </div>
      )}

      {formOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div className="absolute inset-0 bg-black/70 backdrop-blur-sm" onClick={() => setFormOpen(false)} />
          <div className="relative w-full max-w-md rounded-xl border border-border bg-surface p-5 shadow-2xl">
            <div className="mb-4 flex items-center justify-between">
              <h2 className="text-[14px] font-semibold text-primary">{t("plugins.form.addTitle")}</h2>
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
                <label className="mb-1 block text-[13px] font-medium text-tertiary">
                  {t("plugins.form.marketplaceKey")}
                </label>
                <input
                  type="text"
                  value={form.marketplaceKey}
                  onChange={(e) => setForm((f) => ({ ...f, marketplaceKey: e.target.value }))}
                  placeholder={t("plugins.form.marketplaceKeyPlaceholder")}
                  className="app-input w-full bg-background"
                />
              </div>
              <div>
                <label className="mb-1 block text-[13px] font-medium text-tertiary">
                  {t("plugins.form.marketplaceUrl")}
                </label>
                <input
                  type="text"
                  value={form.marketplaceUrl}
                  onChange={(e) => setForm((f) => ({ ...f, marketplaceUrl: e.target.value }))}
                  placeholder={t("plugins.form.marketplaceUrlPlaceholder")}
                  className="app-input w-full bg-background"
                />
              </div>
              <div>
                <label className="mb-1 block text-[13px] font-medium text-tertiary">
                  {t("plugins.form.pluginId")}
                </label>
                <input
                  type="text"
                  value={form.pluginId}
                  onChange={(e) => setForm((f) => ({ ...f, pluginId: e.target.value }))}
                  placeholder={t("plugins.form.pluginIdPlaceholder")}
                  className="app-input w-full bg-background"
                />
              </div>
              <div>
                <label className="mb-1 block text-[13px] font-medium text-tertiary">
                  {t("plugins.form.name")}
                </label>
                <input
                  type="text"
                  value={form.name}
                  onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                  placeholder={t("plugins.form.namePlaceholder")}
                  className="app-input w-full bg-background"
                />
              </div>
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
        title={t("plugins.remove")}
        message={t("plugins.deleteConfirm", { name: deleteTarget?.name || deleteTarget?.plugin_id || "" })}
        onClose={() => setDeleteTarget(null)}
        onConfirm={handleDelete}
      />
    </div>
  );
}
