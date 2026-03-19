import { useState, useEffect, type FormEvent } from "react";
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
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { VAULT_COLORS } from "@/lib/colors";
import { cn } from "@/lib/utils";
import { vaultApi } from "@/api/vaults";
import { toApiError } from "@/lib/api-error";
import type { VaultListItem } from "@/types/vault";
import { Building2, User } from "lucide-react";

interface CreateVaultDialogProps {
  open: boolean;
  onClose: () => void;
  onCreated: (vault: VaultListItem) => void;
  /** Pre-select vault type code (e.g. "personal", "organization") */
  initialType?: string;
}

const VAULT_TYPES = [
  { code: "organization", icon: Building2, labelKey: "vault.typeOrganization", hintKey: "vault.typeOrganizationHint" },
  { code: "personal", icon: User, labelKey: "vault.typePersonal", hintKey: "vault.typePersonalHint" },
] as const;

export function CreateVaultDialog({ open, onClose, onCreated, initialType }: CreateVaultDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [vaultType, setVaultType] = useState<string>(initialType || "organization");
  const [colorCode, setColorCode] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  // Sync initialType when dialog opens
  useEffect(() => {
    if (open && initialType) {
      setVaultType(initialType);
    }
  }, [open, initialType]);

  const resetForm = () => {
    setName("");
    setDescription("");
    setVaultType(initialType || "organization");
    setColorCode(0);
    setError("");
  };

  const handleClose = () => {
    resetForm();
    onClose();
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    const trimmedName = name.trim();
    if (!trimmedName) {
      setError(t("vault.errorNameRequired"));
      return;
    }

    setLoading(true);
    setError("");

    try {
      const vault = await vaultApi.create({
        name: trimmedName,
        description: description.trim() || undefined,
        vaultTypeId: vaultType,
        colorCode,
      });
      onCreated({
        id: vault.id,
        name: vault.name,
        description: vault.description,
        vaultTypeId: vault.vaultTypeId,
        colorCode: vault.colorCode,
        itemCount: vault.itemCount,
        folderCount: vault.folderCount,
        createdAt: vault.createdAt,
      });
      handleClose();
    } catch (err) {
      const apiErr = toApiError(err);
      if (apiErr.code === "VAULT_NAME_TAKEN") {
        setError(t("vault.errorNameTaken"));
      } else {
        setError(t("vault.errorCreateFailed"));
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onClose={handleClose}>
      <form onSubmit={handleSubmit}>
        <DialogHeader>
          <DialogTitle>{t("vault.createTitle")}</DialogTitle>
          <DialogDescription>{t("vault.createDescription")}</DialogDescription>
        </DialogHeader>

        <DialogContent>
          <div className="space-y-4">
            {/* Vault type — only show picker when type is not pre-selected */}
            {!initialType && (
              <div className="space-y-2">
                <Label>{t("vault.typeLabel")}</Label>
                <div className="grid grid-cols-2 gap-2">
                  {VAULT_TYPES.map((vt) => {
                    const active = vaultType === vt.code;
                    const Icon = vt.icon;
                    return (
                      <button
                        key={vt.code}
                        type="button"
                        onClick={() => setVaultType(vt.code)}
                        className={cn(
                          "flex items-center gap-3 rounded-md border p-3 text-left transition-colors",
                          active
                            ? "border-primary bg-primary/5"
                            : "border-border hover:border-muted-foreground/40",
                        )}
                      >
                        <Icon className={cn("h-5 w-5 shrink-0", active ? "text-primary" : "text-muted-foreground")} />
                        <div className="min-w-0">
                          <p className={cn("text-sm font-medium", active && "text-primary")}>{t(vt.labelKey)}</p>
                          <p className="text-xs text-muted-foreground truncate">{t(vt.hintKey)}</p>
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>
            )}

            {/* Name */}
            <div className="space-y-2">
              <Label htmlFor="vault-name">{t("vault.nameLabel")}</Label>
              <Input
                id="vault-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={t("vault.namePlaceholder")}
                maxLength={100}
                autoFocus
              />
            </div>

            {/* Description */}
            <div className="space-y-2">
              <Label htmlFor="vault-desc">
                {t("vault.descriptionLabel")}{" "}
                <span className="text-muted-foreground font-normal">
                  ({t("common.optional")})
                </span>
              </Label>
              <Textarea
                id="vault-desc"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder={t("vault.descriptionPlaceholder")}
                maxLength={500}
                rows={3}
              />
            </div>

            {/* Color picker */}
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
                  >
                    <span className={`block h-full w-full rounded-full ${c.bg}`} />
                  </button>
                ))}
              </div>
            </div>

            {error && (
              <p className="text-sm text-destructive">{error}</p>
            )}
          </div>
        </DialogContent>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={handleClose} disabled={loading}>
            {t("vault.cancel")}
          </Button>
          <Button type="submit" disabled={loading || !name.trim()}>
            {loading && <Spinner size="sm" />}
            {t("vault.create")}
          </Button>
        </DialogFooter>
      </form>
    </Dialog>
  );
}
