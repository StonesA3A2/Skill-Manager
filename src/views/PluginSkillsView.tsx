import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Blocks, Loader2 } from "lucide-react";
import * as api from "../lib/tauri";
import type { Plugin, PluginSkillPreview } from "../lib/tauri";
import { getErrorMessage } from "../lib/error";

interface PluginWithSkills {
  plugin: Plugin;
  skills: PluginSkillPreview[] | null;
  error: string | null;
}

/// Read-only: skills bundled inside active plugins aren't managed by Skill
/// Manager (no symlink/central-copy, no delete/update here) — this view just
/// answers "what do my enabled plugins bring with them", mirroring the
/// per-plugin expander on the Plugins page but flattened across all of them.
export function PluginSkillsView() {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<PluginWithSkills[]>([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const plugins = (await api.listPlugins()).filter((p) => p.enabled);
      const results = await Promise.all(
        plugins.map(async (plugin): Promise<PluginWithSkills> => {
          try {
            const skills = await api.listPluginSkills(plugin.id);
            return { plugin, skills, error: null };
          } catch (err) {
            return { plugin, skills: null, error: getErrorMessage(err, t("common.error")) };
          }
        })
      );
      setEntries(results);
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    load();
  }, [load]);

  if (loading) {
    return (
      <div className="app-panel flex items-center justify-center gap-2 p-10 text-[13px] text-muted">
        <Loader2 className="h-4 w-4 animate-spin" />
        {t("plugins.loadingSkills")}
      </div>
    );
  }

  if (entries.length === 0) {
    return (
      <div className="app-panel flex flex-col items-center gap-3 p-10 text-center">
        <Blocks className="h-8 w-8 text-faint" />
        <p className="text-[13px] text-muted">{t("mySkills.pluginSkills.noActivePlugins")}</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {entries.map(({ plugin, skills, error }) => (
        <section key={plugin.id} className="app-panel overflow-hidden">
          <div className="flex items-center gap-3 border-b border-border-subtle px-4 py-3.5">
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border bg-surface-hover">
              <Blocks className="h-4 w-4 text-accent" />
            </div>
            <div className="min-w-0 flex-1">
              <p className="truncate text-[13px] font-medium text-secondary">
                {plugin.name || plugin.plugin_id}
              </p>
              <p className="truncate text-[12px] text-muted">
                {plugin.plugin_id}@{plugin.marketplace_key}
                {skills && ` — ${t("mySkills.pluginSkills.count", { count: skills.length })}`}
              </p>
            </div>
          </div>
          <div className="px-4 py-3">
            {error ? (
              <p className="text-[13px] text-red-400">{error}</p>
            ) : skills && skills.length > 0 ? (
              <ul className="grid gap-1.5 sm:grid-cols-2">
                {skills.map((skill) => (
                  <li key={skill.rel_path} className="text-[13px]">
                    <span className="font-medium text-secondary">{skill.name}</span>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="text-[13px] text-muted">{t("plugins.noSkillsFound")}</p>
            )}
          </div>
        </section>
      ))}
    </div>
  );
}
