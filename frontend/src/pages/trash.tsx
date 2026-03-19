import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Trash2, RotateCcw, AlertTriangle, Globe, Key } from "lucide-react";
import { AppLayout } from "@/components/layout/app-layout";
import { Spinner } from "@/components/ui/spinner";
import { trashApi } from "@/api/vaults";
import { getColor } from "@/lib/colors";
import type { TrashListEntry } from "@/types/vault";

// Event to notify sidebar badge
const TRASH_CHANGED = "lockso:trash-changed";
export function notifyTrashChanged() {
  window.dispatchEvent(new Event(TRASH_CHANGED));
}

export function TrashPage() {
  const { t } = useTranslation();
  const [items, setItems] = useState<TrashListEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [emptyConfirm, setEmptyConfirm] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [restoringId, setRestoringId] = useState<string | null>(null);
  const [emptying, setEmptying] = useState(false);

  const load = useCallback(async () => {
    try {
      const data = await trashApi.list();
      setItems(data);
    } catch {
      setItems([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleRestore = async (id: string) => {
    setRestoringId(id);
    try {
      await trashApi.restore(id);
      setItems((prev) => prev.filter((i) => i.id !== id));
      notifyTrashChanged();
    } catch {
      // silently fail
    } finally {
      setRestoringId(null);
    }
  };

  const handlePermanentDelete = async (id: string) => {
    setDeletingId(id);
    try {
      await trashApi.permanentDelete(id);
      setItems((prev) => prev.filter((i) => i.id !== id));
      notifyTrashChanged();
    } catch {
      // silently fail
    } finally {
      setDeletingId(null);
    }
  };

  const handleEmptyTrash = async () => {
    setEmptying(true);
    try {
      await trashApi.empty();
      setItems([]);
      notifyTrashChanged();
    } catch {
      // silently fail
    } finally {
      setEmptying(false);
      setEmptyConfirm(false);
    }
  };

  const formatDeletedAt = (iso: string) => {
    const date = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return t("trash.today");
    if (diffDays === 1) return t("trash.yesterday");
    if (diffDays < 30) return t("trash.daysAgo", { count: diffDays });
    return date.toLocaleDateString();
  };

  return (
    <AppLayout>
      <div className="flex items-center justify-between gap-4">
        <h1 className="text-2xl font-bold tracking-tight shrink-0">
          {t("trash.title")}
        </h1>
        {items.length > 0 && (
          <button
            onClick={() => setEmptyConfirm(true)}
            disabled={emptying}
            className="rounded-md bg-destructive px-3 py-1.5 text-sm font-medium text-white hover:bg-destructive/90 disabled:opacity-50"
          >
            {t("trash.emptyAll")}
          </button>
        )}
      </div>

      {items.length > 0 && (
        <div className="mt-3 flex items-center gap-2 rounded-md border border-amber-500/30 bg-amber-500/5 px-3 py-2 text-xs text-amber-700 dark:text-amber-400">
          <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
          <span>{t("trash.retentionWarning")}</span>
        </div>
      )}

      {loading ? (
        <div className="mt-16 flex justify-center">
          <Spinner size="lg" />
        </div>
      ) : items.length === 0 ? (
        <div className="mt-16 flex flex-col items-center justify-center text-center">
          <div className="flex h-16 w-16 items-center justify-center rounded-full bg-muted">
            <Trash2 className="h-8 w-8 text-muted-foreground" />
          </div>
          <h2 className="mt-4 text-lg font-semibold text-foreground">
            {t("trash.emptyTitle")}
          </h2>
          <p className="mt-2 max-w-sm text-sm text-muted-foreground">
            {t("trash.emptyDescription")}
          </p>
        </div>
      ) : (
        <div className="mt-6 space-y-1">
          {items.map((item) => {
            const color = getColor(item.colorCode);
            const isRestoring = restoringId === item.id;
            const isDeleting = deletingId === item.id;

            return (
              <div
                key={item.id}
                className="flex w-full items-center gap-3 rounded-md px-3 py-2.5 transition-colors hover:bg-muted group"
              >
                <div
                  className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-lg ${color.bg}`}
                >
                  {item.url ? (
                    <Globe className={`h-4 w-4 ${color.text}`} />
                  ) : (
                    <Key className={`h-4 w-4 ${color.text}`} />
                  )}
                </div>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium text-foreground">
                    {item.name}
                  </p>
                  <p className="truncate text-xs text-muted-foreground">
                    {item.login && <span>{item.login}</span>}
                    {item.login && item.vaultName && <span> · </span>}
                    <span className="text-muted-foreground/60">{item.vaultName}</span>
                  </p>
                </div>
                <span className="text-[10px] text-muted-foreground/60 shrink-0">
                  {formatDeletedAt(item.deletedAt)}
                </span>
                <div className="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    onClick={() => handleRestore(item.id)}
                    disabled={isRestoring || isDeleting}
                    className="rounded-md p-1.5 text-foreground hover:bg-primary/10 hover:text-primary disabled:opacity-50"
                    title={t("trash.restore")}
                  >
                    <RotateCcw className="h-3.5 w-3.5" />
                  </button>
                  <button
                    onClick={() => handlePermanentDelete(item.id)}
                    disabled={isRestoring || isDeleting}
                    className="rounded-md p-1.5 text-destructive hover:bg-destructive/10 disabled:opacity-50"
                    title={t("trash.permanentDelete")}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Empty trash confirmation dialog */}
      {emptyConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div className="w-full max-w-sm rounded-lg border border-border bg-card p-6 shadow-lg">
            <h3 className="text-lg font-semibold">{t("trash.emptyAll")}</h3>
            <p className="mt-2 text-sm text-muted-foreground">
              {t("trash.emptyConfirm", { count: items.length })}
            </p>
            <div className="mt-4 flex justify-end gap-3">
              <button
                onClick={() => setEmptyConfirm(false)}
                disabled={emptying}
                className="rounded-md border border-border px-3 py-1.5 text-sm font-medium text-foreground hover:bg-muted disabled:opacity-50"
              >
                {t("vault.cancel")}
              </button>
              <button
                onClick={handleEmptyTrash}
                disabled={emptying}
                className="rounded-md bg-destructive px-3 py-1.5 text-sm font-medium text-white hover:bg-destructive/90 disabled:opacity-50"
              >
                {emptying ? t("trash.emptying") : t("trash.emptyAll")}
              </button>
            </div>
          </div>
        </div>
      )}
    </AppLayout>
  );
}
