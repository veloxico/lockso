import { useEffect } from "react";
import { useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import { LocksoLogo } from "@/components/lockso-logo";
import { Spinner } from "@/components/ui/spinner";
import { AlertCircle } from "lucide-react";
import { useHealthCheck } from "@/hooks/use-health-check";
import { useAuthStore } from "@/stores/auth";

/**
 * Initial loading screen.
 * Checks system health and routes to the appropriate page:
 * - Not bootstrapped → /wizard (first-time setup)
 * - Not authenticated → /login
 * - Authenticated → / (dashboard)
 */
export function LoadingPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { data, error, isLoading } = useHealthCheck();
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);

  useEffect(() => {
    if (isLoading || !data) return;

    if (data.status !== "healthy") {
      // Stay on loading page with degraded warning
      return;
    }

    if (!data.isBootstrapped) {
      navigate("/wizard", { replace: true });
      return;
    }

    if (isAuthenticated) {
      navigate("/", { replace: true });
    } else {
      navigate("/login", { replace: true });
    }
  }, [data, isLoading, isAuthenticated, navigate]);

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background">
      <LocksoLogo size="lg" className="mb-6" />

      {isLoading && (
        <div className="flex flex-col items-center gap-3">
          <Spinner size="lg" />
          <p className="text-sm text-muted-foreground">
            {t("loading.checking")}
          </p>
        </div>
      )}

      {error && (
        <div className="flex flex-col items-center gap-3 text-center">
          <AlertCircle className="h-10 w-10 text-destructive" />
          <div>
            <p className="font-medium text-foreground">
              {t("loading.connectionFailed")}
            </p>
            <p className="mt-1 text-sm text-muted-foreground">
              {t("loading.connectionFailedHint")}
            </p>
          </div>
          <button
            onClick={() => window.location.reload()}
            className="mt-2 text-sm font-medium text-primary hover:underline"
          >
            {t("loading.retry")}
          </button>
        </div>
      )}

      {data && data.status === "degraded" && (
        <div className="flex flex-col items-center gap-3 text-center">
          <AlertCircle className="h-10 w-10 text-warning" />
          <div>
            <p className="font-medium text-foreground">
              {t("loading.degraded")}
            </p>
            <p className="mt-1 text-sm text-muted-foreground">
              {t("loading.degradedHint")}
            </p>
          </div>
          <div className="mt-2 space-y-1 text-xs text-muted-foreground">
            {data.database.status !== "ok" && (
              <p>{t("loading.serviceFailed", { service: "Database" })}</p>
            )}
            {data.redis.status !== "ok" && (
              <p>{t("loading.serviceFailed", { service: "Redis" })}</p>
            )}
            {data.storage.status !== "ok" && (
              <p>{t("loading.serviceFailed", { service: "Storage" })}</p>
            )}
          </div>
          <button
            onClick={() => window.location.reload()}
            className="mt-2 text-sm font-medium text-primary hover:underline"
          >
            {t("loading.retry")}
          </button>
        </div>
      )}
    </div>
  );
}
