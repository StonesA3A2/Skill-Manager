import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { getSettings } from "../lib/tauri";
import zh from "./zh.json";
import zhTW from "./zh-TW.json";
import en from "./en.json";
import de from "./de.json";
import es from "./es.json";
import fr from "./fr.json";
import hu from "./hu.json";
import pt from "./pt.json";
import nl from "./nl.json";
import it from "./it.json";

const LANGUAGE_STORAGE_KEY = "language";
const SUPPORTED_LANGUAGES = ["zh", "zh-TW", "en", "de", "es", "fr", "hu", "pt", "nl", "it"] as const;
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
      es: { translation: es },
      fr: { translation: fr },
      hu: { translation: hu },
      pt: { translation: pt },
      nl: { translation: nl },
      it: { translation: it },
    },
    lng,
    fallbackLng: "zh",
    interpolation: { escapeValue: false },
  });
})();

export default i18n;
