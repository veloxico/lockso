import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import { Clock, Globe, Key } from "lucide-react";
import { AppLayout } from "@/components/layout/app-layout";
import { Spinner } from "@/components/ui/spinner";
import { SearchBar } from "@/components/search-bar";
import { getColor } from "@/lib/colors";
import { itemApi } from "@/api/vaults";
import type { ItemListEntry } from "@/types/vault";

export function RecentPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [items, setItems] = useState<ItemListEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const loadRecent = useCallback(async () => {
    try {
      const data = await itemApi.recent();
      setItems(data);
    } catch {
      setItems([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadRecent();
  }, [loadRecent]);

  return (
    <AppLayout>
      <div className="flex items-center justify-between gap-4">
        <h1 className="text-2xl font-bold tracking-tight shrink-0">
          {t("nav.recent")}
        </h1>
        <SearchBar />
      </div>

      {loading ? (
        <div className="mt-16 flex justify-center">
          <Spinner size="lg" />
        </div>
      ) : items.length === 0 ? (
        <div className="mt-16 flex flex-col items-center justify-center text-center">
          <div className="flex h-16 w-16 items-center justify-center rounded-full bg-muted">
            <Clock className="h-8 w-8 text-muted-foreground" />
          </div>
          <h2 className="mt-4 text-lg font-semibold text-foreground">
            {t("recent.emptyTitle")}
          </h2>
          <p className="mt-2 max-w-sm text-sm text-muted-foreground">
            {t("recent.emptyDescription")}
          </p>
        </div>
      ) : (
        <div className="mt-6 space-y-1">
          {items.map((item) => {
            const color = getColor(item.colorCode);
            return (
              <button
                key={item.id}
                onClick={() => navigate(`/vault/${item.vaultId}?item=${item.id}`)}
                className="flex w-full items-center gap-3 rounded-md px-3 py-3 text-left transition-colors hover:bg-muted"
              >
                <div
                  className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-lg ${color.bg}`}
                >
                  {item.url ? (
                    <Globe className={`h-5 w-5 ${color.text}`} />
                  ) : (
                    <Key className={`h-5 w-5 ${color.text}`} />
                  )}
                </div>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium text-foreground">
                    {item.name}
                  </p>
                  <p className="truncate text-xs text-muted-foreground">
                    {item.login}
                  </p>
                </div>
                <span className="text-xs text-muted-foreground shrink-0">
                  {new Date(item.updatedAt).toLocaleDateString()}
                </span>
              </button>
            );
          })}
        </div>
      )}
    </AppLayout>
  );
}
