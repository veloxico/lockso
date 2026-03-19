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
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { folderApi } from "@/api/vaults";
import { toApiError } from "@/lib/api-error";

interface FolderDialogProps {
  open: boolean;
  mode: "create" | "rename";
  vaultId: string;
  parentFolderId?: string;
  /** For rename mode */
  folderId?: string;
  initialName?: string;
  onClose: () => void;
  onSuccess: () => void;
}

export function FolderDialog({
  open,
  mode,
  vaultId,
  parentFolderId,
  folderId,
  initialName,
  onClose,
  onSuccess,
}: FolderDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (open) {
      setName(mode === "rename" && initialName ? initialName : "");
      setError("");
    }
  }, [open, mode, initialName]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) {
      setError(t("folder.errorNameRequired"));
      return;
    }

    setLoading(true);
    setError("");

    try {
      if (mode === "create") {
        await folderApi.create({
          name: trimmed,
          vaultId,
          parentFolderId: parentFolderId || undefined,
        });
      } else if (folderId) {
        await folderApi.update(folderId, { name: trimmed });
      }
      onSuccess();
      onClose();
    } catch (err) {
      const apiErr = toApiError(err);
      if (apiErr.code === "FOLDER_NAME_TAKEN") {
        setError(t("folder.errorNameTaken"));
      } else {
        setError(t("folder.errorSaveFailed"));
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose}>
      <form onSubmit={handleSubmit}>
        <DialogHeader>
          <DialogTitle>
            {mode === "create" ? t("folder.createTitle") : t("folder.renameTitle")}
          </DialogTitle>
        </DialogHeader>

        <DialogContent>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="folder-name">{t("folder.nameLabel")}</Label>
              <Input
                id="folder-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={t("folder.namePlaceholder")}
                maxLength={100}
                autoFocus
              />
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
            {mode === "create" ? t("folder.create") : t("folder.save")}
          </Button>
        </DialogFooter>
      </form>
    </Dialog>
  );
}
