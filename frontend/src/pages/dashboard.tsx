import { useTranslation } from "react-i18next";
import { Lock, Plus } from "lucide-react";
import { AppLayout } from "@/components/layout/app-layout";
import { Button } from "@/components/ui/button";

/**
 * Dashboard / Vaults page.
 * Phase 1 placeholder — will be populated with vault list in Phase 2.
 */
export function DashboardPage() {
  const { t } = useTranslation();

  return (
    <AppLayout>
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">
          {t("dashboard.title")}
        </h1>
        <Button disabled>
          <Plus className="h-4 w-4" />
          {t("dashboard.createVault")}
        </Button>
      </div>

      {/* Empty state */}
      <div className="mt-16 flex flex-col items-center justify-center text-center">
        <div className="flex h-16 w-16 items-center justify-center rounded-full bg-muted">
          <Lock className="h-8 w-8 text-muted-foreground" />
        </div>
        <h2 className="mt-4 text-lg font-semibold text-foreground">
          {t("dashboard.emptyTitle")}
        </h2>
        <p className="mt-2 max-w-sm text-sm text-muted-foreground">
          {t("dashboard.emptyDescription")}
        </p>
      </div>
    </AppLayout>
  );
}
