import { useState, useEffect, useCallback, useMemo } from "react";
import { useParams, useNavigate, useSearchParams } from "react-router";
import { useTranslation } from "react-i18next";
import {
  Plus,
  Users,
  Upload,
  Download,
  MoreHorizontal,
  PanelLeftOpen,
} from "lucide-react";
import { AppLayout, useSidebar } from "@/components/layout/app-layout";
import { Breadcrumbs } from "@/components/breadcrumbs";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { ItemList } from "@/components/item/item-list";
import { ItemDetailPanel } from "@/components/item/item-detail-panel";
import { ItemFormDialog } from "@/components/item/item-form-dialog";
import { SnapshotViewer } from "@/components/item/snapshot-viewer";
import { ShareVaultDialog } from "@/components/vault/share-vault-dialog";
import { ImportDialog } from "@/components/vault/import-dialog";
import { ExportDialog } from "@/components/vault/export-dialog";
import { vaultApi, folderApi, itemApi } from "@/api/vaults";
import type {
  VaultView,
  FolderTreeNode,
  ItemListEntry,
  ItemView,
} from "@/types/vault";

export function VaultDetailPage() {
  const { t } = useTranslation();
  const { vaultId } = useParams<{ vaultId: string }>();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();

  // Data
  const [vault, setVault] = useState<VaultView | null>(null);
  const [folders, setFolders] = useState<FolderTreeNode[]>([]);
  const [items, setItems] = useState<ItemListEntry[]>([]);
  const [loadingVault, setLoadingVault] = useState(true);
  const [loadingItems, setLoadingItems] = useState(false);

  // Selection from URL
  const selectedFolderId = searchParams.get("folder");
  const selectedItemId = searchParams.get("item");

  // Dialogs
  const [itemDialogOpen, setItemDialogOpen] = useState(false);
  const [itemDialogMode, setItemDialogMode] = useState<"create" | "edit">(
    "create",
  );
  const [editingItem, setEditingItem] = useState<ItemView | null>(null);
  const [deletingItem, setDeletingItem] = useState<ItemView | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState("");
  const [snapshotItem, setSnapshotItem] = useState<ItemView | null>(null);
  const [shareOpen, setShareOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [exportOpen, setExportOpen] = useState(false);
  const [showMoreMenu, setShowMoreMenu] = useState(false);

  // Load vault + folders
  const loadVault = useCallback(async () => {
    if (!vaultId) return;
    setLoadingVault(true);
    try {
      const [vaultData, foldersData] = await Promise.all([
        vaultApi.get(vaultId),
        folderApi.tree(vaultId),
      ]);
      setVault(vaultData);
      setFolders(foldersData);
    } catch {
      navigate("/");
    } finally {
      setLoadingVault(false);
    }
  }, [vaultId, navigate]);

  // Load items
  const loadItems = useCallback(async () => {
    if (!vaultId) return;
    setLoadingItems(true);
    try {
      const data = await itemApi.list(
        vaultId,
        selectedFolderId || undefined,
      );
      setItems(data);
    } catch {
      setItems([]);
    } finally {
      setLoadingItems(false);
    }
  }, [vaultId, selectedFolderId]);

  useEffect(() => {
    loadVault();
  }, [loadVault]);

  useEffect(() => {
    loadItems();
  }, [loadItems]);

  // Find selected folder name for breadcrumbs
  const selectedFolderName = useMemo(() => {
    if (!selectedFolderId) return null;
    const find = (nodes: FolderTreeNode[]): string | null => {
      for (const n of nodes) {
        if (n.id === selectedFolderId) return n.name;
        const found = find(n.children);
        if (found) return found;
      }
      return null;
    };
    return find(folders);
  }, [selectedFolderId, folders]);

  // Breadcrumb items
  const breadcrumbItems = useMemo(() => {
    const crumbs = [];
    if (vault) {
      crumbs.push({
        label: vault.name,
        href: `/vault/${vault.id}`,
        icon: "vault" as const,
        colorCode: vault.colorCode,
      });
    }
    if (selectedFolderName) {
      crumbs.push({
        label: selectedFolderName,
        icon: "folder" as const,
      });
    }
    return crumbs;
  }, [vault, selectedFolderName]);

  // Item handlers
  const handleSelectItem = (id: string) => {
    const params = new URLSearchParams(searchParams);
    params.set("item", id);
    setSearchParams(params);
  };

  const handleCreateItem = () => {
    setItemDialogMode("create");
    setEditingItem(null);
    setItemDialogOpen(true);
  };

  const handleEditItem = (item: ItemView) => {
    setItemDialogMode("edit");
    setEditingItem(item);
    setItemDialogOpen(true);
  };

  const handleDeleteItem = async () => {
    if (!deletingItem) return;
    setDeleting(true);
    setDeleteError("");
    try {
      await itemApi.delete(deletingItem.id);
      setDeletingItem(null);
      if (selectedItemId === deletingItem.id) {
        const params = new URLSearchParams(searchParams);
        params.delete("item");
        setSearchParams(params);
      }
      await loadItems();
      // Notify sidebar trash badge
      window.dispatchEvent(new Event("lockso:trash-changed"));
    } catch (err: unknown) {
      const msg = err && typeof err === "object" && "message" in err
        ? String((err as { message: string }).message)
        : t("item.deleteFailed");
      setDeleteError(msg);
    } finally {
      setDeleting(false);
    }
  };

  const handleItemSaved = async () => {
    await loadItems();
  };

  if (loadingVault) {
    return (
      <AppLayout fullHeight>
        <div className="flex h-full items-center justify-center">
          <Spinner size="lg" />
        </div>
      </AppLayout>
    );
  }

  if (!vault) return null;

  return (
    <AppLayout fullHeight>
      {/* Toolbar */}
      <VaultToolbar
        breadcrumbItems={breadcrumbItems}
        onCreateItem={handleCreateItem}
        showMoreMenu={showMoreMenu}
        setShowMoreMenu={setShowMoreMenu}
        onShare={() => { setShareOpen(true); setShowMoreMenu(false); }}
        onImport={() => { setImportOpen(true); setShowMoreMenu(false); }}
        onExport={() => { setExportOpen(true); setShowMoreMenu(false); }}
      />

      {/* Main content: items list | item detail — fills remaining space */}
      <div className="flex flex-1 overflow-hidden">
        {/* Item list */}
        <div className="w-72 shrink-0 border-r border-border flex flex-col overflow-hidden bg-card/50">
          <div className="shrink-0 border-b border-border px-3 py-2">
            <p className="text-[11px] font-medium text-muted-foreground">
              {selectedFolderName
                ? `${selectedFolderName} — ${t("item.count", { count: items.length })}`
                : t("item.count", { count: items.length })}
            </p>
          </div>
          <div className="flex-1 overflow-y-auto p-1">
            <ItemList
              items={items}
              selectedItemId={selectedItemId}
              onSelectItem={handleSelectItem}
              loading={loadingItems}
            />
          </div>
        </div>

        {/* Item detail */}
        <div className="flex-1 overflow-hidden bg-background">
          <ItemDetailPanel
            itemId={selectedItemId}
            onEdit={handleEditItem}
            onDelete={setDeletingItem}
            onViewSnapshots={setSnapshotItem}
            onFavoriteToggled={loadItems}
          />
        </div>
      </div>

      {/* Dialogs */}
      {vaultId && (
        <ItemFormDialog
          open={itemDialogOpen}
          mode={itemDialogMode}
          vaultId={vaultId}
          folderId={selectedFolderId}
          item={editingItem}
          onClose={() => {
            setItemDialogOpen(false);
            setEditingItem(null);
          }}
          onSuccess={handleItemSaved}
        />
      )}

      {deletingItem && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div className="w-full max-w-sm rounded-lg border border-border bg-card p-6 shadow-lg">
            <h3 className="text-lg font-semibold">{t("item.deleteTitle")}</h3>
            <p className="mt-2 text-sm text-muted-foreground">
              {t("item.deleteWarning", { name: deletingItem.name })}
            </p>
            {deleteError && (
              <p className="mt-2 text-sm text-destructive">{deleteError}</p>
            )}
            <div className="mt-4 flex justify-end gap-3">
              <Button
                variant="outline"
                onClick={() => setDeletingItem(null)}
                disabled={deleting}
              >
                {t("vault.cancel")}
              </Button>
              <Button
                variant="destructive"
                onClick={handleDeleteItem}
                disabled={deleting}
              >
                {deleting && <Spinner size="sm" />}
                {t("item.deleteConfirm")}
              </Button>
            </div>
          </div>
        </div>
      )}

      <SnapshotViewer
        open={!!snapshotItem}
        itemId={snapshotItem?.id || null}
        itemName={snapshotItem?.name || ""}
        onClose={() => setSnapshotItem(null)}
        onReverted={() => {
          loadItems();
          setSnapshotItem(null);
        }}
      />

      <ShareVaultDialog
        open={shareOpen}
        onClose={() => setShareOpen(false)}
        vaultId={vaultId!}
        vaultName={vault.name}
        creatorId={vault.creatorId}
      />

      <ImportDialog
        open={importOpen}
        onClose={() => setImportOpen(false)}
        vaultId={vaultId!}
        vaultName={vault.name}
        onSuccess={() => loadItems()}
      />

      <ExportDialog
        open={exportOpen}
        onClose={() => setExportOpen(false)}
        vaultId={vaultId!}
        vaultName={vault.name}
      />
    </AppLayout>
  );
}

// ─── Vault Toolbar ───

function VaultToolbar({
  breadcrumbItems,
  onCreateItem,
  showMoreMenu,
  setShowMoreMenu,
  onShare,
  onImport,
  onExport,
}: {
  breadcrumbItems: Array<{ label: string; href?: string; icon: "vault" | "folder"; colorCode?: number }>;
  onCreateItem: () => void;
  showMoreMenu: boolean;
  setShowMoreMenu: (v: boolean) => void;
  onShare: () => void;
  onImport: () => void;
  onExport: () => void;
}) {
  const { t } = useTranslation();
  const { collapsed, toggle } = useSidebar();

  return (
    <div className="flex items-center gap-3 border-b border-border bg-card/50 px-4 py-2 shrink-0">
      {/* Sidebar expand button (only when collapsed) */}
      {collapsed && (
        <button
          onClick={toggle}
          className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          title={t("sidebar.expand")}
        >
          <PanelLeftOpen className="h-4 w-4" />
        </button>
      )}

      {/* Breadcrumbs */}
      <div className="flex-1 min-w-0">
        <Breadcrumbs items={breadcrumbItems} />
      </div>

      {/* Actions */}
      <div className="flex items-center gap-1.5 shrink-0">
        <Button onClick={onCreateItem} size="sm" className="h-7 text-xs px-2.5">
          <Plus className="h-3.5 w-3.5" />
          {t("item.create")}
        </Button>

        <div className="relative">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowMoreMenu(!showMoreMenu)}
            className="h-7 w-7 p-0"
          >
            <MoreHorizontal className="h-4 w-4" />
          </Button>

          {showMoreMenu && (
            <>
              <div
                className="fixed inset-0 z-10"
                onClick={() => setShowMoreMenu(false)}
              />
              <div className="absolute right-0 top-full z-20 mt-1 min-w-[160px] rounded-md border border-border bg-card py-1 shadow-lg">
                <button
                  onClick={onShare}
                  className="flex w-full items-center gap-2 px-3 py-1.5 text-xs text-foreground hover:bg-muted"
                >
                  <Users className="h-3.5 w-3.5" />
                  {t("sharing.share")}
                </button>
                <button
                  onClick={onImport}
                  className="flex w-full items-center gap-2 px-3 py-1.5 text-xs text-foreground hover:bg-muted"
                >
                  <Upload className="h-3.5 w-3.5" />
                  {t("importExport.import")}
                </button>
                <button
                  onClick={onExport}
                  className="flex w-full items-center gap-2 px-3 py-1.5 text-xs text-foreground hover:bg-muted"
                >
                  <Download className="h-3.5 w-3.5" />
                  {t("importExport.export")}
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
