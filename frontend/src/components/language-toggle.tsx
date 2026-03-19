import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

const languages = [
  { code: "en", label: "EN" },
  { code: "ru", label: "RU" },
];

export function LanguageToggle() {
  const { i18n } = useTranslation();

  return (
    <div className="flex items-center rounded-lg border border-border bg-muted/50 p-0.5">
      {languages.map((lang) => {
        const isActive = i18n.language.startsWith(lang.code);
        return (
          <button
            key={lang.code}
            onClick={() => i18n.changeLanguage(lang.code)}
            className={cn(
              "flex items-center justify-center rounded-md px-2 py-1 text-xs font-medium transition-all",
              isActive
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground",
            )}
          >
            {lang.label}
          </button>
        );
      })}
    </div>
  );
}
