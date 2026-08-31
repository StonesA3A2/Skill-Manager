import { useState } from "react";
import { useTranslation } from "react-i18next";
import { MySkills } from "./MySkills";
import { PluginSkillsView } from "./PluginSkillsView";

type LibraryTab = "solo" | "plugin";

/// Thin wrapper so `MySkills` (the existing, large "Solo Skills" library
/// view — left entirely untouched here) and `PluginSkillsView` (new,
/// read-only) can share the /my-skills route behind a tab without either
/// one needing to know about the other.
export function Library() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<LibraryTab>("solo");

  return (
    <>
      <div className="flex justify-end px-4 pt-3">
        <div className="inline-flex items-center gap-1 rounded-lg border border-border bg-surface p-1">
          <button
            type="button"
            onClick={() => setTab("solo")}
            className={`rounded-md px-3 py-1.5 text-[13px] font-medium transition-colors ${
              tab === "solo" ? "bg-accent-bg text-accent-light" : "text-tertiary hover:bg-surface-hover"
            }`}
          >
            {t("mySkills.pluginSkills.soloTab")}
          </button>
          <button
            type="button"
            onClick={() => setTab("plugin")}
            className={`rounded-md px-3 py-1.5 text-[13px] font-medium transition-colors ${
              tab === "plugin" ? "bg-accent-bg text-accent-light" : "text-tertiary hover:bg-surface-hover"
            }`}
          >
            {t("mySkills.pluginSkills.pluginTab")}
          </button>
        </div>
      </div>
      {tab === "solo" ? <MySkills /> : <div className="app-page">{<PluginSkillsView />}</div>}
    </>
  );
}
