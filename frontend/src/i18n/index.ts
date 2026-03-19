import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import en from "./locales/en.json";
import ru from "./locales/ru.json";

const resources = {
  en: { translation: en },
  ru: { translation: ru },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    supportedLngs: ["en", "ru"],
    fallbackLng: "en",
    interpolation: {
      // React already escapes rendered output — safe to disable double-escaping.
      // If values are ever used outside React (e.g. document.title), sanitize manually.
      escapeValue: false,
    },
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: "i18nextLng",
      caches: ["localStorage"],
      // Ensure exact match: "ru-RU" → "ru", "en-US" → "en"
      convertDetectedLanguage: (lng: string): string => lng.split("-")[0] ?? lng,
    },
  });

export default i18n;
