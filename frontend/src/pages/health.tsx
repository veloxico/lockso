import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import {
  ShieldCheck,
  ShieldAlert,
  AlertTriangle,
  Copy,
  Clock,
  Globe,
  Key,
  RefreshCw,
  ShieldX,
} from "lucide-react";
import { AppLayout } from "@/components/layout/app-layout";
import { Spinner } from "@/components/ui/spinner";
import { healthApi } from "@/api/vaults";
import { getColor } from "@/lib/colors";
import type { HealthReport, HealthItem } from "@/types/vault";

type Tab = "all" | "weak" | "reused" | "old" | "breached";

export function HealthPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [report, setReport] = useState<HealthReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<Tab>("all");
  const [error, setError] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(false);
    try {
      const data = await healthApi.report();
      setReport(data);
    } catch {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const filteredItems = report
    ? report.items.filter((item) => {
        switch (activeTab) {
          case "weak":
            return item.isWeak;
          case "reused":
            return item.isReused;
          case "old":
            return item.isOld;
          case "breached":
            return item.isBreached;
          default:
            return item.isWeak || item.isReused || item.isOld || item.isBreached;
        }
      })
    : [];

  const scoreColor =
    !report
      ? "text-muted-foreground"
      : report.score >= 80
        ? "text-emerald-500"
        : report.score >= 60
          ? "text-amber-500"
          : "text-destructive";

  const scoreLabel =
    !report
      ? ""
      : report.score >= 80
        ? t("health.scoreGood")
        : report.score >= 60
          ? t("health.scoreFair")
          : t("health.scorePoor");

  return (
    <AppLayout>
      <div className="flex items-center justify-between gap-4">
        <h1 className="text-2xl font-bold tracking-tight shrink-0">
          {t("health.title")}
        </h1>
        {report && (
          <button
            onClick={load}
            disabled={loading}
            className="flex items-center gap-1.5 rounded-md border border-border px-3 py-1.5 text-sm font-medium text-foreground hover:bg-muted disabled:opacity-50"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
            {t("health.refresh")}
          </button>
        )}
      </div>

      {loading && !report ? (
        <div className="mt-16 flex flex-col items-center justify-center gap-3">
          <Spinner size="lg" />
          <p className="text-sm text-muted-foreground">{t("health.analyzing")}</p>
        </div>
      ) : error ? (
        <div className="mt-16 flex flex-col items-center justify-center text-center">
          <ShieldAlert className="h-12 w-12 text-destructive" />
          <p className="mt-4 text-sm text-muted-foreground">{t("health.error")}</p>
          <button
            onClick={load}
            className="mt-3 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
          >
            {t("health.retry")}
          </button>
        </div>
      ) : report && report.totalItems === 0 ? (
        <div className="mt-16 flex flex-col items-center justify-center text-center">
          <div className="flex h-16 w-16 items-center justify-center rounded-full bg-muted">
            <ShieldCheck className="h-8 w-8 text-muted-foreground" />
          </div>
          <h2 className="mt-4 text-lg font-semibold">{t("health.noItems")}</h2>
          <p className="mt-2 max-w-sm text-sm text-muted-foreground">
            {t("health.noItemsDescription")}
          </p>
        </div>
      ) : report ? (
        <div className="mt-4 space-y-6">
          {/* Score card */}
          <div className="flex flex-col sm:flex-row gap-4">
            <div className="flex-1 rounded-lg border border-border bg-card p-5">
              <div className="flex items-center gap-4">
                <div className="relative">
                  <svg viewBox="0 0 36 36" className="h-20 w-20 -rotate-90">
                    <path
                      d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="3"
                      className="text-muted/30"
                    />
                    <path
                      d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="3"
                      strokeDasharray={`${report.score}, 100`}
                      className={scoreColor}
                      strokeLinecap="round"
                    />
                  </svg>
                  <div className="absolute inset-0 flex items-center justify-center">
                    <span className={`text-xl font-bold ${scoreColor}`}>
                      {report.score}
                    </span>
                  </div>
                </div>
                <div>
                  <p className={`text-lg font-semibold ${scoreColor}`}>{scoreLabel}</p>
                  <p className="text-sm text-muted-foreground">
                    {t("health.totalItems", { count: report.totalItems })}
                  </p>
                </div>
              </div>
            </div>

            {/* Issue cards */}
            <div className="flex flex-1 gap-3">
              <IssueCard
                icon={<ShieldX className="h-4 w-4 text-rose-600" />}
                count={report.breachedCount}
                label={t("health.breached")}
                active={activeTab === "breached"}
                onClick={() => setActiveTab(activeTab === "breached" ? "all" : "breached")}
              />
              <IssueCard
                icon={<AlertTriangle className="h-4 w-4 text-red-500" />}
                count={report.weakCount}
                label={t("health.weak")}
                active={activeTab === "weak"}
                onClick={() => setActiveTab(activeTab === "weak" ? "all" : "weak")}
              />
              <IssueCard
                icon={<Copy className="h-4 w-4 text-orange-500" />}
                count={report.reusedCount}
                label={t("health.reused")}
                active={activeTab === "reused"}
                onClick={() => setActiveTab(activeTab === "reused" ? "all" : "reused")}
              />
              <IssueCard
                icon={<Clock className="h-4 w-4 text-amber-500" />}
                count={report.oldCount}
                label={t("health.old")}
                active={activeTab === "old"}
                onClick={() => setActiveTab(activeTab === "old" ? "all" : "old")}
              />
            </div>
          </div>

          {/* Items list */}
          {filteredItems.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <ShieldCheck className="h-10 w-10 text-emerald-500" />
              <p className="mt-3 text-sm font-medium text-foreground">
                {t("health.allGood")}
              </p>
              <p className="mt-1 text-xs text-muted-foreground">
                {t("health.allGoodDescription")}
              </p>
            </div>
          ) : (
            <div className="space-y-1">
              <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider px-1 mb-2">
                {t("health.issuesFound", { count: filteredItems.length })}
              </p>
              {filteredItems.map((item) => (
                <HealthItemRow
                  key={item.id}
                  item={item}
                  onClick={() => navigate(`/vault/${item.vaultId}?item=${item.id}`)}
                />
              ))}
            </div>
          )}
        </div>
      ) : null}
    </AppLayout>
  );
}

function IssueCard({
  icon,
  count,
  label,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  count: number;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex-1 rounded-lg border p-3 text-left transition-colors ${
        active
          ? "border-primary bg-primary/5"
          : "border-border bg-card hover:bg-muted/50"
      }`}
    >
      <div className="flex items-center gap-2">
        {icon}
        <span className="text-2xl font-bold text-foreground">{count}</span>
      </div>
      <p className="mt-1 text-[11px] text-muted-foreground">{label}</p>
    </button>
  );
}

function HealthItemRow({
  item,
  onClick,
}: {
  item: HealthItem;
  onClick: () => void;
}) {
  const { t } = useTranslation();
  const color = getColor(item.colorCode);

  const strengthLabel = [
    t("health.veryWeak"),
    t("health.weakLabel"),
    t("health.fair"),
    t("health.good"),
    t("health.strong"),
  ][item.strength] || t("health.veryWeak");

  const strengthColor = [
    "text-red-500",
    "text-red-400",
    "text-amber-500",
    "text-emerald-400",
    "text-emerald-500",
  ][item.strength] || "text-red-500";

  return (
    <button
      onClick={onClick}
      className="flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left transition-colors hover:bg-muted group"
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

      {/* Issue badges */}
      <div className="flex items-center gap-1.5 shrink-0">
        {item.isBreached && (
          <span className="inline-flex items-center gap-0.5 rounded-full bg-rose-600/10 px-1.5 py-0.5 text-[10px] font-medium text-rose-600 dark:text-rose-400">
            <ShieldX className="h-2.5 w-2.5" />
            {t("health.breachedLabel", { count: item.breachCount })}
          </span>
        )}
        {item.isWeak && (
          <span className="inline-flex items-center gap-0.5 rounded-full bg-red-500/10 px-1.5 py-0.5 text-[10px] font-medium text-red-500">
            <AlertTriangle className="h-2.5 w-2.5" />
            {strengthLabel}
          </span>
        )}
        {item.isReused && (
          <span className="inline-flex items-center gap-0.5 rounded-full bg-orange-500/10 px-1.5 py-0.5 text-[10px] font-medium text-orange-500">
            <Copy className="h-2.5 w-2.5" />
            {t("health.reused")}
          </span>
        )}
        {item.isOld && (
          <span className="inline-flex items-center gap-0.5 rounded-full bg-amber-500/10 px-1.5 py-0.5 text-[10px] font-medium text-amber-600 dark:text-amber-400">
            <Clock className="h-2.5 w-2.5" />
            {item.passwordAgeDays}{t("health.daysShort")}
          </span>
        )}
        {!item.isWeak && !item.isBreached && (
          <span className={`text-[10px] font-medium ${strengthColor}`}>
            {strengthLabel}
          </span>
        )}
      </div>
    </button>
  );
}
