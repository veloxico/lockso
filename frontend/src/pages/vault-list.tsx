import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import { Plus, Lock } from "lucide-react";
import { AppLayout } from "@/components/layout/app-layout";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { SearchBar } from "@/components/search-bar";
import { VaultCard } from "@/components/vault/vault-card";
import { CreateVaultDialog } from "@/components/vault/create-vault-dialog";
import { EditVaultDialog } from "@/components/vault/edit-vault-dialog";
import { DeleteVaultDialog } from "@/components/vault/delete-vault-dialog";
import { vaultApi } from "@/api/vaults";
import type { VaultListItem } from "@/types/vault";

export function VaultListPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const [vaults, setVaults] = useState<VaultListItem[]>([]);
  const [loading, setLoading] = useState(true);

  // Dialog state
  const [createOpen, setCreateOpen] = useState(false);
  const [editVault, setEditVault] = useState<VaultListItem | null>(null);
  const [deleteVault, setDeleteVault] = useState<VaultListItem | null>(null);

  const loadVaults = useCallback(async () => {
    try {
      const data = await vaultApi.list();
      setVaults(data);
    } catch {
      // Keep current list on error
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadVaults();
  }, [loadVaults]);

  const handleCreated = (vault: VaultListItem) => {
    setVaults((prev) => [vault, ...prev]);
    window.dispatchEvent(new Event("lockso:reload-vaults"));
  };

  const handleUpdated = (updated: VaultListItem) => {
    setVaults((prev) =>
      prev.map((v) => (v.id === updated.id ? updated : v)),
    );
    window.dispatchEvent(new Event("lockso:reload-vaults"));
  };

  const handleDeleted = (id: string) => {
    setVaults((prev) => prev.filter((v) => v.id !== id));
    window.dispatchEvent(new Event("lockso:reload-vaults"));
  };

  return (
    <AppLayout>
      {/* Header */}
      <div className="flex items-center justify-between gap-4">
        <h1 className="text-2xl font-bold tracking-tight shrink-0">
          {t("dashboard.title")}
        </h1>
        <SearchBar />
        <Button onClick={() => setCreateOpen(true)} className="shrink-0">
          <Plus className="h-4 w-4" />
          {t("dashboard.createVault")}
        </Button>
      </div>

      {/* Content */}
      {loading ? (
        <div className="mt-16 flex justify-center">
          <Spinner size="lg" />
        </div>
      ) : vaults.length === 0 ? (
        /* Empty state */
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
          <Button className="mt-6" onClick={() => setCreateOpen(true)}>
            <Plus className="h-4 w-4" />
            {t("dashboard.createVault")}
          </Button>
        </div>
      ) : (
        /* Vault grid */
        <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {vaults.map((vault) => (
            <VaultCard
              key={vault.id}
              vault={vault}
              onClick={() => navigate(`/vault/${vault.id}`)}
              onEdit={() => setEditVault(vault)}
              onDelete={() => setDeleteVault(vault)}
            />
          ))}
        </div>
      )}

      {/* Dialogs */}
      <CreateVaultDialog
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        onCreated={handleCreated}
      />

      <EditVaultDialog
        open={!!editVault}
        vault={editVault}
        onClose={() => setEditVault(null)}
        onUpdated={handleUpdated}
      />

      <DeleteVaultDialog
        open={!!deleteVault}
        vault={deleteVault}
        onClose={() => setDeleteVault(null)}
        onDeleted={handleDeleted}
      />
    </AppLayout>
  );
}
