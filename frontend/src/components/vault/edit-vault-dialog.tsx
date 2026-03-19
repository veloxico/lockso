import { useState, useEffect, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogContent,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { VAULT_COLORS } from "@/lib/colors";
import { cn } from "@/lib/utils";
import { vaultApi } from "@/api/vaults";
import { toApiError } from "@/lib/api-error";
import type { VaultListItem } from "@/types/vault";

interface EditVaultDialogProps {
  open: boolean;
  vault: VaultListItem | null;
  onClose: () => void;
  onUpdated: (vault: VaultListItem) => void;
}

export function EditVaultDialog({ open, vault, onClose, onUpdated }: EditVaultDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [colorCode, setColorCode] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (vault) {
      setName(vault.name);
      setDescription(vault.description || "");
      setColorCode(vault.colorCode);
      setError("");
    }
  }, [vault]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!vault) return;

    const trimmedName = name.trim();
    if (!trimmedName) {
      setError(t("vault.errorNameRequired"));
      return;
    }

    setLoading(true);
    setError("");

    try {
      const updated = await vaultApi.update(vault.id, {
        name: trimmedName,
        description: description.trim() || undefined,
        colorCode,
      });
      onUpdated({
        id: updated.id,
        name: updated.name,
        description: updated.description,
        vaultTypeId: updated.vaultTypeId,
        colorCode: updated.colorCode,
        itemCount: updated.itemCount,
        folderCount: updated.folderCount,
        createdAt: updated.createdAt,
      });
      onClose();
    } catch (err) {
      const apiErr = toApiError(err);
      if (apiErr.code === "VAULT_NAME_TAKEN") {
        setError(t("vault.errorNameTaken"));
      } else {
        setError(t("vault.errorUpdateFailed"));
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose}>
      <form onSubmit={handleSubmit}>
        <DialogHeader>
          <DialogTitle>{t("vault.editTitle")}</DialogTitle>
        </DialogHeader>

        <DialogContent>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="edit-vault-name">{t("vault.nameLabel")}</Label>
              <Input
                id="edit-vault-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                maxLength={100}
                autoFocus
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="edit-vault-desc">
                {t("vault.descriptionLabel")}{" "}
                <span className="text-muted-foreground font-normal">
                  ({t("common.optional")})
                </span>
              </Label>
              <Textarea
                id="edit-vault-desc"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                maxLength={500}
                rows={3}
              />
            </div>

            <div className="space-y-2">
              <Label>{t("vault.colorLabel")}</Label>
              <div className="flex flex-wrap gap-2">
                {VAULT_COLORS.map((c, i) => (
                  <button
                    key={i}
                    type="button"
                    onClick={() => setColorCode(i)}
                    className={cn(
                      "h-8 w-8 rounded-full transition-all",
                      c.bg,
                      colorCode === i
                        ? `ring-2 ${c.ring} ring-offset-2 ring-offset-background`
                        : "hover:scale-110",
                    )}
                    title={c.label}
                  />
                ))}
              </div>
            </div>

            {error && <p className="text-sm text-destructive">{error}</p>}
          </div>
        </DialogContent>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={onClose} disabled={loading}>
            {t("vault.cancel")}
          </Button>
          <Button type="submit" disabled={loading || !name.trim()}>
            {loading && <Spinner size="sm" />}
            {t("vault.save")}
          </Button>
        </DialogFooter>
      </form>
    </Dialog>
  );
}
