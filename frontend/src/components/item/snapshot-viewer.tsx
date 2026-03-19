import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { History, RotateCcw, Eye, EyeOff, ChevronLeft } from "lucide-react";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogContent,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Spinner } from "@/components/ui/spinner";
import { itemApi } from "@/api/vaults";
import type { SnapshotListEntry, SnapshotView } from "@/types/vault";

interface SnapshotViewerProps {
  open: boolean;
  itemId: string | null;
  itemName: string;
  onClose: () => void;
  onReverted: () => void;
}

export function SnapshotViewer({
  open,
  itemId,
  itemName,
  onClose,
  onReverted,
}: SnapshotViewerProps) {
  const { t } = useTranslation();
  const [snapshots, setSnapshots] = useState<SnapshotListEntry[]>([]);
  const [selectedSnapshot, setSelectedSnapshot] = useState<SnapshotView | null>(null);
  const [loading, setLoading] = useState(false);
  const [_loadingDetail, setLoadingDetail] = useState(false);
  const [reverting, setReverting] = useState(false);
  const [showPassword, setShowPassword] = useState(false);

  const loadSnapshots = useCallback(async () => {
    if (!itemId) return;
    setLoading(true);
    try {
      const data = await itemApi.listSnapshots(itemId);
      setSnapshots(data);
    } catch {
      setSnapshots([]);
    } finally {
      setLoading(false);
    }
  }, [itemId]);

  useEffect(() => {
    if (open) {
      setSelectedSnapshot(null);
      setShowPassword(false);
      loadSnapshots();
    }
  }, [open, loadSnapshots]);

  const viewSnapshot = async (snapshotId: string) => {
    if (!itemId) return;
    setLoadingDetail(true);
    setShowPassword(false);
    try {
      const data = await itemApi.getSnapshot(itemId, snapshotId);
      setSelectedSnapshot(data);
    } catch {
      setSelectedSnapshot(null);
    } finally {
      setLoadingDetail(false);
    }
  };

  const handleRevert = async () => {
    if (!itemId || !selectedSnapshot) return;
    setReverting(true);
    try {
      await itemApi.revertToSnapshot(itemId, selectedSnapshot.id);
      onReverted();
      onClose();
    } catch {
      // Ignore
    } finally {
      setReverting(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogHeader>
        <DialogTitle>
          <div className="flex items-center gap-2">
            <History className="h-5 w-5" />
            {t("snapshot.title")}
          </div>
        </DialogTitle>
        <DialogDescription>
          {t("snapshot.description", { name: itemName })}
        </DialogDescription>
      </DialogHeader>

      <DialogContent>
        <div className="max-h-[50vh] overflow-y-auto">
          {loading ? (
            <div className="flex justify-center py-8">
              <Spinner size="md" />
            </div>
          ) : selectedSnapshot ? (
            /* Snapshot detail view */
            <div className="space-y-4">
              <button
                onClick={() => setSelectedSnapshot(null)}
                className="flex items-center gap-1 text-sm text-primary hover:underline"
              >
                <ChevronLeft className="h-4 w-4" />
                {t("snapshot.backToList")}
              </button>

              <div className="space-y-3">
                <div>
                  <p className="text-xs text-muted-foreground">{t("item.nameLabel")}</p>
                  <p className="text-sm font-medium">{selectedSnapshot.name}</p>
                </div>
                {selectedSnapshot.login && (
                  <div>
                    <p className="text-xs text-muted-foreground">{t("item.loginLabel")}</p>
                    <p className="text-sm">{selectedSnapshot.login}</p>
                  </div>
                )}
                {selectedSnapshot.password && (
                  <div>
                    <p className="text-xs text-muted-foreground">{t("item.passwordLabel")}</p>
                    <div className="flex items-center gap-2">
                      <code className="text-sm font-mono">
                        {showPassword ? selectedSnapshot.password : "••••••••••••"}
                      </code>
                      <button
                        onClick={() => setShowPassword(!showPassword)}
                        className="text-muted-foreground hover:text-foreground"
                      >
                        {showPassword ? (
                          <EyeOff className="h-3.5 w-3.5" />
                        ) : (
                          <Eye className="h-3.5 w-3.5" />
                        )}
                      </button>
                    </div>
                  </div>
                )}
                {selectedSnapshot.url && (
                  <div>
                    <p className="text-xs text-muted-foreground">{t("item.urlLabel")}</p>
                    <p className="text-sm">{selectedSnapshot.url}</p>
                  </div>
                )}
                {selectedSnapshot.description && (
                  <div>
                    <p className="text-xs text-muted-foreground">{t("item.descriptionLabel")}</p>
                    <p className="text-sm whitespace-pre-wrap">{selectedSnapshot.description}</p>
                  </div>
                )}
                {selectedSnapshot.tags.length > 0 && (
                  <div>
                    <p className="text-xs text-muted-foreground">{t("item.tagsLabel")}</p>
                    <div className="flex flex-wrap gap-1 mt-1">
                      {selectedSnapshot.tags.map((tag) => (
                        <Badge key={tag} variant="secondary">{tag}</Badge>
                      ))}
                    </div>
                  </div>
                )}
                <div className="text-xs text-muted-foreground">
                  {t("snapshot.createdAt", {
                    date: new Date(selectedSnapshot.createdAt).toLocaleString(),
                  })}
                </div>
              </div>
            </div>
          ) : snapshots.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-8 text-center">
              <History className="h-8 w-8 text-muted-foreground" />
              <p className="mt-2 text-sm text-muted-foreground">
                {t("snapshot.empty")}
              </p>
            </div>
          ) : (
            /* Snapshot list */
            <div className="space-y-1">
              {snapshots.map((snap) => (
                <button
                  key={snap.id}
                  onClick={() => viewSnapshot(snap.id)}
                  className="flex w-full items-center justify-between rounded-md px-3 py-2.5 text-left hover:bg-muted transition-colors"
                >
                  <div>
                    <p className="text-sm font-medium text-foreground">{snap.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {snap.login && `${snap.login} · `}
                      {new Date(snap.createdAt).toLocaleString()}
                    </p>
                  </div>
                  <ChevronLeft className="h-4 w-4 rotate-180 text-muted-foreground" />
                </button>
              ))}
            </div>
          )}
        </div>
      </DialogContent>

      {selectedSnapshot && (
        <DialogFooter>
          <Button variant="outline" onClick={() => setSelectedSnapshot(null)}>
            {t("vault.cancel")}
          </Button>
          <Button onClick={handleRevert} disabled={reverting}>
            {reverting ? <Spinner size="sm" /> : <RotateCcw className="h-4 w-4" />}
            {t("snapshot.revert")}
          </Button>
        </DialogFooter>
      )}
    </Dialog>
  );
}
