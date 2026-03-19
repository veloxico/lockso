import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogContent,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { vaultApi } from "@/api/vaults";
import type { VaultListItem } from "@/types/vault";

interface DeleteVaultDialogProps {
  open: boolean;
  vault: VaultListItem | null;
  onClose: () => void;
  onDeleted: (id: string) => void;
}

export function DeleteVaultDialog({ open, vault, onClose, onDeleted }: DeleteVaultDialogProps) {
  const { t } = useTranslation();
  const [confirmation, setConfirmation] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const canDelete = vault ? confirmation === vault.name : false;

  const handleClose = () => {
    setConfirmation("");
    setError("");
    onClose();
  };

  const handleDelete = async () => {
    if (!vault || !canDelete) return;

    setLoading(true);
    setError("");

    try {
      await vaultApi.delete(vault.id);
      onDeleted(vault.id);
      handleClose();
    } catch {
      setError(t("vault.errorDeleteFailed"));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onClose={handleClose}>
      <DialogHeader>
        <DialogTitle>{t("vault.deleteTitle")}</DialogTitle>
        <DialogDescription>{t("vault.deleteWarning")}</DialogDescription>
      </DialogHeader>

      <DialogContent>
        <div className="space-y-4">
          <p className="text-sm text-muted-foreground">
            {t("vault.deleteConfirmHint", { name: vault?.name || "" })}
          </p>

          <div className="space-y-2">
            <Label htmlFor="delete-confirm">{t("vault.deleteConfirmLabel")}</Label>
            <Input
              id="delete-confirm"
              value={confirmation}
              onChange={(e) => setConfirmation(e.target.value)}
              placeholder={vault?.name || ""}
              autoFocus
            />
          </div>

          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>
      </DialogContent>

      <DialogFooter>
        <Button type="button" variant="outline" onClick={handleClose} disabled={loading}>
          {t("vault.cancel")}
        </Button>
        <Button
          variant="destructive"
          onClick={handleDelete}
          disabled={!canDelete || loading}
        >
          {loading && <Spinner size="sm" />}
          {t("vault.deleteConfirm")}
        </Button>
      </DialogFooter>
    </Dialog>
  );
}
