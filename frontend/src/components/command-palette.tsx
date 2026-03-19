import { useState, useEffect, useRef, useCallback } from "react";
import { useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import {
  Search,
  Clock,
  Star,
  Settings,
  Key,
  Globe,
  ArrowRight,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { getColor } from "@/lib/colors";
import { itemApi } from "@/api/vaults";
import type { ItemListEntry } from "@/types/vault";

interface CommandItem {
  id: string;
  type: "action" | "item";
  icon: typeof Search;
  iconColor?: string;
  label: string;
  hint?: string;
  action: () => void;
}

export function CommandPalette() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ItemListEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Global keyboard shortcut: Cmd+K / Ctrl+K
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setOpen(true);
      }
      if (e.key === "Escape" && open) {
        e.preventDefault();
        close();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open]);

  // Focus input when opened
  useEffect(() => {
    if (open) {
      setQuery("");
      setResults([]);
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  // Clean up debounce
  useEffect(() => {
    return () => {
      if (debounceRef.current !== null) clearTimeout(debounceRef.current);
    };
  }, []);

  const close = () => {
    setOpen(false);
    setQuery("");
    setResults([]);
  };

  const doSearch = useCallback(async (q: string) => {
    if (q.trim().length < 2) {
      setResults([]);
      return;
    }
    setLoading(true);
    try {
      const data = await itemApi.search({ query: q.trim() });
      setResults(data);
    } catch {
      setResults([]);
    } finally {
      setLoading(false);
    }
  }, []);

  const handleChange = (value: string) => {
    setQuery(value);
    setSelectedIndex(0);
    if (debounceRef.current !== null) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => doSearch(value), 200);
  };

  // Static quick actions
  const quickActions: CommandItem[] = [
    {
      id: "nav-recent",
      type: "action",
      icon: Clock,
      label: t("nav.recent"),
      action: () => {
        navigate("/recent");
        close();
      },
    },
    {
      id: "nav-favorites",
      type: "action",
      icon: Star,
      label: t("nav.favorites"),
      action: () => {
        navigate("/favorites");
        close();
      },
    },
    {
      id: "nav-settings",
      type: "action",
      icon: Settings,
      label: t("nav.settings"),
      action: () => {
        navigate("/settings");
        close();
      },
    },
  ];

  // Build visible list
  const hasQuery = query.trim().length >= 2;
  const searchItems: CommandItem[] = results.map((item) => {
    const color = getColor(item.colorCode);
    return {
      id: item.id,
      type: "item" as const,
      icon: item.url ? Globe : Key,
      iconColor: color.text,
      label: item.name,
      hint: item.login || item.url || undefined,
      action: () => {
        navigate(`/vault/${item.vaultId}?item=${item.id}`);
        close();
      },
    };
  });

  const visibleItems = hasQuery
    ? searchItems
    : quickActions.filter((a) =>
        a.label.toLowerCase().includes(query.toLowerCase()),
      );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, visibleItems.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter" && visibleItems[selectedIndex]) {
      e.preventDefault();
      visibleItems[selectedIndex].action();
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-[100]">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={close}
      />

      {/* Dialog */}
      <div className="relative mx-auto mt-[15vh] w-full max-w-lg px-4">
        <div className="overflow-hidden rounded-xl border border-border bg-card shadow-2xl">
          {/* Search input */}
          <div className="flex items-center gap-3 border-b border-border px-4">
            <Search className="h-4 w-4 shrink-0 text-muted-foreground" />
            <input
              ref={inputRef}
              value={query}
              onChange={(e) => handleChange(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={t("commandPalette.placeholder")}
              className="flex-1 bg-transparent py-3.5 text-sm text-foreground outline-none placeholder:text-muted-foreground"
              autoComplete="off"
              spellCheck={false}
            />
            <kbd className="hidden sm:inline-flex h-5 items-center rounded border border-border bg-muted px-1.5 text-[10px] font-medium text-muted-foreground">
              ESC
            </kbd>
          </div>

          {/* Results */}
          <div className="max-h-80 overflow-y-auto p-1.5">
            {loading && (
              <div className="px-3 py-4 text-center text-sm text-muted-foreground">
                {t("search.searching")}
              </div>
            )}

            {!loading && hasQuery && searchItems.length === 0 && (
              <div className="px-3 py-4 text-center text-sm text-muted-foreground">
                {t("search.noResults")}
              </div>
            )}

            {!loading && !hasQuery && (
              <div className="px-2 pb-1 pt-1">
                <p className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground/60 px-1">
                  {t("commandPalette.quickActions")}
                </p>
              </div>
            )}

            {!loading &&
              visibleItems.map((item, idx) => (
                <button
                  key={item.id}
                  onClick={item.action}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  className={cn(
                    "flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors",
                    idx === selectedIndex
                      ? "bg-primary/10 text-foreground"
                      : "text-muted-foreground hover:bg-muted",
                  )}
                >
                  <item.icon
                    className={cn(
                      "h-4 w-4 shrink-0",
                      item.iconColor || "text-muted-foreground",
                    )}
                  />
                  <span className="min-w-0 flex-1 truncate text-left">
                    {item.label}
                  </span>
                  {item.hint && (
                    <span className="truncate text-xs text-muted-foreground/60 max-w-[150px]">
                      {item.hint}
                    </span>
                  )}
                  {idx === selectedIndex && (
                    <ArrowRight className="h-3 w-3 shrink-0 text-primary" />
                  )}
                </button>
              ))}
          </div>

          {/* Footer hint */}
          <div className="border-t border-border px-4 py-2 flex items-center gap-4 text-[11px] text-muted-foreground/60">
            <span>
              <kbd className="font-medium">↑↓</kbd>{" "}
              {t("commandPalette.navigate")}
            </span>
            <span>
              <kbd className="font-medium">↵</kbd>{" "}
              {t("commandPalette.open")}
            </span>
            <span>
              <kbd className="font-medium">esc</kbd>{" "}
              {t("commandPalette.close")}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
