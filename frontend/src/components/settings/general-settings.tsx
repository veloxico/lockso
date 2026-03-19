import { useState, useEffect, useMemo, useRef, useCallback, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { Save, Search, Check, ChevronsUpDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { settingsApi } from "@/api/admin";
import type { InterfaceSettings } from "@/types/admin";

/* ── Timezone helpers ── */

function getTimezones(): string[] {
  try {
    return (Intl as unknown as { supportedValuesOf(key: string): string[] }).supportedValuesOf("timeZone");
  } catch {
    return [
      "UTC",
      "Africa/Cairo", "Africa/Johannesburg", "Africa/Lagos", "Africa/Nairobi",
      "America/Anchorage", "America/Argentina/Buenos_Aires", "America/Bogota",
      "America/Chicago", "America/Denver", "America/Los_Angeles", "America/Mexico_City",
      "America/New_York", "America/Sao_Paulo", "America/Toronto", "America/Vancouver",
      "Asia/Almaty", "Asia/Bangkok", "Asia/Dubai", "Asia/Hong_Kong", "Asia/Istanbul",
      "Asia/Jakarta", "Asia/Karachi", "Asia/Kolkata", "Asia/Krasnoyarsk",
      "Asia/Novosibirsk", "Asia/Seoul", "Asia/Shanghai", "Asia/Singapore",
      "Asia/Tashkent", "Asia/Tehran", "Asia/Tokyo", "Asia/Vladivostok",
      "Asia/Yekaterinburg",
      "Australia/Melbourne", "Australia/Perth", "Australia/Sydney",
      "Europe/Amsterdam", "Europe/Berlin", "Europe/Dublin", "Europe/Helsinki",
      "Europe/Kyiv", "Europe/Lisbon", "Europe/London", "Europe/Madrid",
      "Europe/Minsk", "Europe/Moscow", "Europe/Paris", "Europe/Prague",
      "Europe/Rome", "Europe/Samara", "Europe/Stockholm", "Europe/Vienna",
      "Europe/Warsaw", "Europe/Zurich",
      "Pacific/Auckland", "Pacific/Honolulu",
    ];
  }
}

function getUtcOffset(tz: string): string {
  try {
    const parts = new Intl.DateTimeFormat("en-US", {
      timeZone: tz,
      timeZoneName: "shortOffset",
    }).formatToParts(new Date());
    return parts.find((p) => p.type === "timeZoneName")?.value ?? "";
  } catch {
    return "";
  }
}

/* ── Searchable timezone combobox ── */

function TimezoneCombobox({
  value,
  onChange,
}: {
  value: string;
  onChange: (tz: string) => void;
}) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const allZones = useMemo(() => getTimezones(), []);

  const items = useMemo(() => {
    const q = query.toLowerCase().replace(/\s+/g, "");
    if (!q) return allZones;
    return allZones.filter((tz) => {
      const haystack = tz.toLowerCase().replace(/[/_]/g, "");
      return haystack.includes(q);
    });
  }, [allZones, query]);

  // Close on click outside
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  // Focus search input when opened
  useEffect(() => {
    if (open) {
      setTimeout(() => inputRef.current?.focus(), 0);
    } else {
      setQuery("");
    }
  }, [open]);

  const handleSelect = useCallback(
    (tz: string) => {
      onChange(tz);
      setOpen(false);
    },
    [onChange],
  );

  const offset = getUtcOffset(value);

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm hover:bg-muted/50 transition-colors"
      >
        <span className="truncate">
          {value}{" "}
          <span className="text-muted-foreground">({offset})</span>
        </span>
        <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 text-muted-foreground" />
      </button>

      {open && (
        <div className="absolute z-50 mt-1 w-full rounded-md border border-border bg-card shadow-lg">
          {/* Search field */}
          <div className="flex items-center gap-2 border-b border-border px-3 py-2">
            <Search className="h-4 w-4 text-muted-foreground shrink-0" />
            <input
              ref={inputRef}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t("settings.timezoneSearch")}
              className="w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
            />
          </div>

          {/* Options list */}
          <div className="max-h-56 overflow-y-auto overscroll-contain p-1">
            {items.length === 0 ? (
              <p className="py-4 text-center text-sm text-muted-foreground">
                {t("settings.timezoneNotFound")}
              </p>
            ) : (
              items.map((tz) => {
                const selected = tz === value;
                return (
                  <button
                    key={tz}
                    type="button"
                    onClick={() => handleSelect(tz)}
                    className={`flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm transition-colors ${
                      selected
                        ? "bg-primary/10 text-primary"
                        : "hover:bg-muted"
                    }`}
                  >
                    <Check
                      className={`h-3.5 w-3.5 shrink-0 ${selected ? "opacity-100" : "opacity-0"}`}
                    />
                    <span className="truncate">{tz}</span>
                    <span className="ml-auto text-xs text-muted-foreground shrink-0">
                      {getUtcOffset(tz)}
                    </span>
                  </button>
                );
              })
            )}
          </div>
        </div>
      )}
    </div>
  );
}

/* ── Main component ── */

export function GeneralSettings() {
  const { t, i18n } = useTranslation();
  const [settings, setSettings] = useState<InterfaceSettings>({
    defaultLanguage: "en",
    defaultTimezone: "UTC",
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const data = await settingsApi.get();
      setSettings({
        defaultLanguage: (data.interface as InterfaceSettings).defaultLanguage || "en",
        defaultTimezone: (data.interface as InterfaceSettings).defaultTimezone || "UTC",
      });
    } catch {
      // Use defaults
    } finally {
      setLoading(false);
    }
  };

  const handleLanguageChange = (lang: string) => {
    setSettings({ ...settings, defaultLanguage: lang });
    localStorage.setItem("i18nextLng", lang);
    i18n.changeLanguage(lang);
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError("");
    setSaved(false);

    try {
      await settingsApi.updateCategory("interface", settings as unknown as Record<string, unknown>);
      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch {
      setError(t("settings.errorSaveFailed"));
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex justify-center py-12">
        <Spinner size="md" />
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="max-w-lg space-y-6">
      <div>
        <h2 className="text-lg font-semibold">{t("settings.generalTitle")}</h2>
        <p className="text-sm text-muted-foreground">{t("settings.generalDescription")}</p>
      </div>

      {/* Default language */}
      <div className="space-y-2">
        <Label htmlFor="default-lang">{t("settings.defaultLanguage")}</Label>
        <select
          id="default-lang"
          value={settings.defaultLanguage}
          onChange={(e) => handleLanguageChange(e.target.value)}
          className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
        >
          <option value="en">English</option>
          <option value="ru">Русский</option>
        </select>
      </div>

      {/* Default timezone */}
      <div className="space-y-2">
        <Label>{t("settings.defaultTimezone")}</Label>
        <TimezoneCombobox
          value={settings.defaultTimezone}
          onChange={(tz) => setSettings({ ...settings, defaultTimezone: tz })}
        />
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <Button type="submit" disabled={saving}>
        {saving ? (
          <Spinner size="sm" />
        ) : (
          <Save className="h-4 w-4" />
        )}
        {saved ? t("settings.saved") : t("settings.save")}
      </Button>
    </form>
  );
}
