import { useTranslation } from "react-i18next";
import { Sun, Moon, Monitor } from "lucide-react";
import { useThemeStore, type Theme } from "@/stores/theme";
import { cn } from "@/lib/utils";

const options: { value: Theme; icon: typeof Sun; i18nKey: string }[] = [
  { value: "light", icon: Sun, i18nKey: "theme.light" },
  { value: "system", icon: Monitor, i18nKey: "theme.system" },
  { value: "dark", icon: Moon, i18nKey: "theme.dark" },
];

export function ThemeToggle() {
  const { t } = useTranslation();
  const theme = useThemeStore((s) => s.theme);
  const setTheme = useThemeStore((s) => s.setTheme);

  return (
    <div className="flex items-center rounded-lg border border-border bg-muted/50 p-0.5">
      {options.map((opt) => {
        const isActive = theme === opt.value;
        return (
          <button
            key={opt.value}
            onClick={() => setTheme(opt.value)}
            title={t(opt.i18nKey)}
            aria-label={t(opt.i18nKey)}
            className={cn(
              "flex items-center justify-center rounded-md p-1.5 transition-all",
              isActive
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground",
            )}
          >
            <opt.icon className="h-3.5 w-3.5" />
          </button>
        );
      })}
    </div>
  );
}
