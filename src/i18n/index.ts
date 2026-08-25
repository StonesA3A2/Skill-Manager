import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { getSettings } from "../lib/tauri";
import zh from "./zh.json";
import zhTW from "./zh-TW.json";
import en from "./en.json";
import de from "./de.json";

const LANGUAGE_STORAGE_KEY = "language";
const SUPPORTED_LANGUAGES = ["zh", "zh-TW", "en", "de"] as const;
type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

function isSupportedLanguage(lang: string | null): lang is SupportedLanguage {
  return SUPPORTED_LANGUAGES.includes(lang as SupportedLanguage);
}

function getStoredLanguage(): SupportedLanguage | null {
  const stored = localStorage.getItem(LANGUAGE_STORAGE_KEY);
  return isSupportedLanguage(stored) ? stored : null;
}

export const i18nReady = (async () => {
  const storedLanguage = getStoredLanguage();
  const savedLanguage = await getSettings("language").catch(() => null);
  const lng = isSupportedLanguage(savedLanguage)
    ? savedLanguage
    : storedLanguage || "zh";

  localStorage.setItem(LANGUAGE_STORAGE_KEY, lng);

  await i18n.use(initReactI18next).init({
    resources: {
      zh: { translation: zh },
      "zh-TW": { translation: zhTW },
      en: { translation: en },
      de: { translation: de },
    },
    lng,
    fallbackLng: "zh",
    interpolation: { escapeValue: false },
  });
})();

export default i18n;
