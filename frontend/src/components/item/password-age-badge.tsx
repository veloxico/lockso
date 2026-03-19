import { useTranslation } from "react-i18next";
import { Clock, AlertTriangle } from "lucide-react";

interface PasswordAgeBadgeProps {
  passwordChangedAt: string;
  /** Expiration threshold in days (default: 90) */
  expirationDays?: number;
}

/**
 * Shows password age and warns when it's close to or past expiration.
 */
export function PasswordAgeBadge({
  passwordChangedAt,
  expirationDays = 90,
}: PasswordAgeBadgeProps) {
  const { t } = useTranslation();

  const changedDate = new Date(passwordChangedAt);
  const now = new Date();
  const diffMs = now.getTime() - changedDate.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  const isExpired = diffDays >= expirationDays;
  const isWarning = diffDays >= expirationDays * 0.75; // 75% threshold
  const daysLeft = expirationDays - diffDays;

  let label: string;
  if (diffDays === 0) {
    label = t("passwordAge.today");
  } else if (diffDays < 30) {
    label = t("passwordAge.daysAgo", { count: diffDays });
  } else if (diffDays < 365) {
    const months = Math.floor(diffDays / 30);
    label = t("passwordAge.monthsAgo", { count: months });
  } else {
    const years = Math.floor(diffDays / 365);
    label = t("passwordAge.yearsAgo", { count: years });
  }

  if (isExpired) {
    return (
      <span className="inline-flex items-center gap-1 rounded-full bg-destructive/15 px-2 py-0.5 text-[11px] font-medium text-destructive">
        <AlertTriangle className="h-3 w-3" />
        {t("passwordAge.expired")}
        <span className="text-destructive/70">({label})</span>
      </span>
    );
  }

  if (isWarning) {
    return (
      <span className="inline-flex items-center gap-1 rounded-full bg-warning/15 px-2 py-0.5 text-[11px] font-medium text-warning-foreground">
        <Clock className="h-3 w-3" />
        {t("passwordAge.expiresSoon", { days: daysLeft })}
      </span>
    );
  }

  return (
    <span className="inline-flex items-center gap-1 text-[11px] text-muted-foreground">
      <Clock className="h-3 w-3" />
      {t("passwordAge.changed")} {label}
    </span>
  );
}
