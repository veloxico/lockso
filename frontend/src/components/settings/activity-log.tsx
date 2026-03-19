import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { ChevronLeft, ChevronRight, ScrollText } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { activityApi } from "@/api/activity";
import type { ActivityLogEntry, PaginatedActivityLogs } from "@/types/activity";

const PER_PAGE = 30;

const ACTION_CATEGORIES = [
  { value: "", labelKey: "activityLog.filters.allActions" },
  { value: "auth.", labelKey: "activityLog.categories.auth" },
  { value: "vault.", labelKey: "activityLog.categories.vault" },
  { value: "item.", labelKey: "activityLog.categories.item" },
  { value: "folder.", labelKey: "activityLog.categories.folder" },
  { value: "sharing.", labelKey: "activityLog.categories.sharing" },
  { value: "attachment.", labelKey: "activityLog.categories.attachment" },
  { value: "2fa.", labelKey: "activityLog.categories.twoFactor" },
  { value: "user.", labelKey: "activityLog.categories.user" },
  { value: "settings.", labelKey: "activityLog.categories.settings" },
];

export function ActivityLog() {
  const { t } = useTranslation();
  const [data, setData] = useState<PaginatedActivityLogs | null>(null);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [actionFilter, setActionFilter] = useState("");
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const result = await activityApi.listGlobal({
        page,
        perPage: PER_PAGE,
        action: actionFilter || undefined,
      });
      setData(result);
    } catch {
      setError(t("settings.errorLoadFailed"));
    } finally {
      setLoading(false);
    }
  }, [page, actionFilter, t]);

  useEffect(() => {
    load();
  }, [load]);

  const totalPages = data ? Math.ceil(data.total / PER_PAGE) : 0;

  return (
    <div className="max-w-4xl space-y-4">
      <div className="flex items-center gap-2">
        <ScrollText className="h-5 w-5 text-muted-foreground" />
        <div>
          <h2 className="text-lg font-semibold">{t("activityLog.title")}</h2>
          <p className="text-sm text-muted-foreground">
            {t("activityLog.description")}
          </p>
        </div>
      </div>

      {/* Filter */}
      <div className="flex items-center gap-3">
        <select
          value={actionFilter}
          onChange={(e) => {
            setActionFilter(e.target.value);
            setPage(1);
          }}
          className="h-9 rounded-md border border-input bg-background px-3 text-sm"
        >
          {ACTION_CATEGORIES.map((cat) => (
            <option key={cat.value} value={cat.value}>
              {t(cat.labelKey)}
            </option>
          ))}
        </select>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {loading ? (
        <div className="flex justify-center py-12">
          <Spinner size="md" />
        </div>
      ) : !data || data.data.length === 0 ? (
        <p className="text-sm text-muted-foreground py-8 text-center">
          {t("activityLog.noEntries")}
        </p>
      ) : (
        <>
          <div className="rounded-lg border border-border overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-muted/50 text-xs text-muted-foreground">
                <tr>
                  <th className="text-left px-4 py-2 font-medium">
                    {t("activityLog.columns.time")}
                  </th>
                  <th className="text-left px-4 py-2 font-medium">
                    {t("activityLog.columns.user")}
                  </th>
                  <th className="text-left px-4 py-2 font-medium">
                    {t("activityLog.columns.action")}
                  </th>
                  <th className="text-left px-4 py-2 font-medium">
                    {t("activityLog.columns.ip")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {data.data.map((entry) => (
                  <ActivityRow key={entry.id} entry={entry} />
                ))}
              </tbody>
            </table>
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-between text-sm text-muted-foreground">
              <span>
                {t("activityLog.pagination.showing", {
                  from: (page - 1) * PER_PAGE + 1,
                  to: Math.min(page * PER_PAGE, data.total),
                  total: data.total,
                })}
              </span>
              <div className="flex gap-1">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPage((p) => Math.max(1, p - 1))}
                  disabled={page <= 1}
                >
                  <ChevronLeft className="h-4 w-4" />
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                  disabled={page >= totalPages}
                >
                  <ChevronRight className="h-4 w-4" />
                </Button>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}

function ActivityRow({ entry }: { entry: ActivityLogEntry }) {
  const { t } = useTranslation();

  const actionLabel =
    t(`activityLog.actions.${entry.action}`, { defaultValue: "" }) ||
    entry.action;

  return (
    <tr className="border-t border-border hover:bg-muted/20 transition-colors">
      <td className="px-4 py-2.5 text-xs text-muted-foreground whitespace-nowrap">
        {new Date(entry.createdAt).toLocaleString()}
      </td>
      <td className="px-4 py-2.5">
        {entry.userName || (
          <span className="text-muted-foreground italic">—</span>
        )}
      </td>
      <td className="px-4 py-2.5">
        <span className="inline-flex items-center gap-1.5">
          <ActionBadge action={entry.action} />
          {actionLabel}
        </span>
      </td>
      <td className="px-4 py-2.5 text-xs text-muted-foreground font-mono">
        {entry.clientIp || "—"}
      </td>
    </tr>
  );
}

function ActionBadge({ action }: { action: string }) {
  const category = action.split(".")[0] ?? "";
  const colors: Record<string, string> = {
    auth: "bg-blue-500/15 text-blue-600",
    vault: "bg-emerald-500/15 text-emerald-600",
    item: "bg-violet-500/15 text-violet-600",
    folder: "bg-amber-500/15 text-amber-600",
    sharing: "bg-cyan-500/15 text-cyan-600",
    attachment: "bg-orange-500/15 text-orange-600",
    "2fa": "bg-rose-500/15 text-rose-600",
    user: "bg-pink-500/15 text-pink-600",
    settings: "bg-gray-500/15 text-gray-600",
  };

  return (
    <span
      className={`inline-block rounded px-1.5 py-0.5 text-[10px] font-medium uppercase ${colors[category] || "bg-muted text-muted-foreground"}`}
    >
      {category}
    </span>
  );
}
