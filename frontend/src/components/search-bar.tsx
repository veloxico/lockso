import { useState, useCallback, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router";
import { Search, X, Key, Globe } from "lucide-react";
import { Input } from "@/components/ui/input";
import { getColor } from "@/lib/colors";
import { itemApi } from "@/api/vaults";
import type { ItemListEntry } from "@/types/vault";

interface SearchBarProps {
  vaultId?: string;
}

export function SearchBar({ vaultId }: SearchBarProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ItemListEntry[]>([]);
  const [showResults, setShowResults] = useState(false);
  const [loading, setLoading] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        setShowResults(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  // Clean up debounce timer on unmount
  useEffect(() => {
    return () => {
      if (debounceRef.current !== null) clearTimeout(debounceRef.current);
    };
  }, []);

  const doSearch = useCallback(
    async (q: string) => {
      if (q.trim().length < 2) {
        setResults([]);
        setShowResults(false);
        return;
      }

      setLoading(true);
      try {
        const data = await itemApi.search({
          query: q.trim(),
          vaultId: vaultId || undefined,
        });
        setResults(data);
        setShowResults(true);
      } catch {
        setResults([]);
      } finally {
        setLoading(false);
      }
    },
    [vaultId],
  );

  const handleChange = (value: string) => {
    setQuery(value);
    if (debounceRef.current !== null) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => doSearch(value), 300);
  };

  const handleSelectItem = (item: ItemListEntry) => {
    setQuery("");
    setShowResults(false);
    setResults([]);
    navigate(`/vault/${item.vaultId}?item=${item.id}`);
  };

  const handleClear = () => {
    setQuery("");
    setResults([]);
    setShowResults(false);
  };

  return (
    <div ref={containerRef} className="relative w-full max-w-md">
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          value={query}
          onChange={(e) => handleChange(e.target.value)}
          onFocus={() => {
            if (results.length > 0) setShowResults(true);
          }}
          placeholder={t("search.placeholder")}
          className="pl-9 pr-8"
        />
        {query && (
          <button
            onClick={handleClear}
            className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </div>

      {/* Results dropdown */}
      {showResults && (
        <div className="absolute top-full left-0 right-0 z-50 mt-1 max-h-80 overflow-y-auto rounded-md border border-border bg-card shadow-lg">
          {loading ? (
            <div className="px-4 py-3 text-sm text-muted-foreground">
              {t("search.searching")}
            </div>
          ) : results.length === 0 ? (
            <div className="px-4 py-3 text-sm text-muted-foreground">
              {t("search.noResults")}
            </div>
          ) : (
            results.map((item) => {
              const color = getColor(item.colorCode);
              return (
                <button
                  key={item.id}
                  onClick={() => handleSelectItem(item)}
                  className="flex w-full items-center gap-3 px-3 py-2.5 text-left hover:bg-muted transition-colors"
                >
                  <div
                    className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-md ${color.bg}`}
                  >
                    {item.url ? (
                      <Globe className={`h-4 w-4 ${color.text}`} />
                    ) : (
                      <Key className={`h-4 w-4 ${color.text}`} />
                    )}
                  </div>
                  <div className="min-w-0">
                    <p className="truncate text-sm font-medium text-foreground">
                      {item.name}
                    </p>
                    {item.login && (
                      <p className="truncate text-xs text-muted-foreground">
                        {item.login}
                      </p>
                    )}
                  </div>
                </button>
              );
            })
          )}
        </div>
      )}
    </div>
  );
}
