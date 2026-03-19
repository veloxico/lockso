import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Link, useLocation, useNavigate } from "react-router";
import {
  Star,
  Clock,
  Settings,
  LogOut,
  Shield,
  ShieldCheck,
  Send,
  ChevronRight,
  ChevronDown,
  Folder,
  FolderOpen,
  FolderPlus,
  Lock,
  Plus,
  Search,
  MoreVertical,
  Pencil,
  Trash2,
  PanelLeftClose,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { LocksoLogo } from "@/components/lockso-logo";
import { ThemeToggle } from "@/components/theme-toggle";
import { LanguageToggle } from "@/components/language-toggle";
import { FolderDialog } from "@/components/folder/folder-dialog";
import { CreateVaultDialog } from "@/components/vault/create-vault-dialog";
import { useSidebar } from "./app-layout";
import { useAuthStore } from "@/stores/auth";
import { api } from "@/api/client";
import { vaultApi, folderApi, trashApi } from "@/api/vaults";
import { getColor } from "@/lib/colors";
import type { VaultListItem, FolderTreeNode } from "@/types/vault";

interface VaultTypeGroup {
  id: string;
  code: string;
  name: string;
  vaults: VaultListItem[];
}

// Sidebar-wide event to trigger vault list reload
const RELOAD_EVENT = "lockso:reload-vaults";
export function triggerSidebarReload() {
  window.dispatchEvent(new Event(RELOAD_EVENT));
}

export function Sidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const user = useAuthStore((s) => s.user);
  const logout = useAuthStore((s) => s.logout);
  const { collapsed, toggle } = useSidebar();

  const [groups, setGroups] = useState<VaultTypeGroup[]>([]);
  const [trashCount, setTrashCount] = useState(0);
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(
    new Set(),
  );

  const loadTrashCount = useCallback(async () => {
    try {
      const data = await trashApi.count();
      setTrashCount(data.count);
    } catch {
      // keep current
    }
  }, []);

  const loadVaults = useCallback(async () => {
    try {
      const [vaults, types] = await Promise.all([
        vaultApi.list(),
        api.get<{ id: string; name: string; code: string }[]>("/vault-types"),
      ]);

      const typeMap = new Map<string, VaultTypeGroup>();
      for (const vt of types) {
        typeMap.set(vt.id, {
          id: vt.id,
          code: vt.code,
          name: vt.name,
          vaults: [],
        });
      }
      for (const v of vaults) {
        const group = typeMap.get(v.vaultTypeId);
        if (group) group.vaults.push(v);
      }

      const order = ["organization", "personal", "private_shared"];
      const sorted = [...typeMap.values()].sort((a, b) => {
        const ai = order.indexOf(a.code);
        const bi = order.indexOf(b.code);
        return (ai === -1 ? 99 : ai) - (bi === -1 ? 99 : bi);
      });

      setGroups(sorted);
    } catch {
      // Keep current state
    }
  }, []);

  useEffect(() => {
    loadVaults();
    loadTrashCount();
  }, [loadVaults, loadTrashCount]);

  useEffect(() => {
    const handler = () => loadVaults();
    window.addEventListener(RELOAD_EVENT, handler);
    return () => window.removeEventListener(RELOAD_EVENT, handler);
  }, [loadVaults]);

  // Listen for trash changes (from trash page or delete actions)
  useEffect(() => {
    const handler = () => loadTrashCount();
    window.addEventListener("lockso:trash-changed", handler);
    return () => window.removeEventListener("lockso:trash-changed", handler);
  }, [loadTrashCount]);

  const toggleGroup = (code: string) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(code)) next.delete(code);
      else next.add(code);
      return next;
    });
  };

  const handleLogout = async () => {
    try {
      await api.post("/sessions/logout");
    } catch {
      // Logout even if API call fails
    }
    logout();
  };

  const openCommandPalette = () => {
    window.dispatchEvent(
      new KeyboardEvent("keydown", { key: "k", metaKey: true }),
    );
  };

  const isActive = (path: string) => location.pathname === path;

  const groupLabel = (code: string) => {
    switch (code) {
      case "organization":
        return t("sidebar.orgVaults");
      case "personal":
        return t("sidebar.personalVaults");
      case "private_shared":
        return t("sidebar.sharedVaults");
      default:
        return code;
    }
  };

  return (
    <aside
      className={cn(
        "flex flex-col border-r border-border bg-card transition-all duration-200 ease-in-out overflow-hidden",
        collapsed ? "w-0 border-r-0" : "w-64",
      )}
    >
      <div className="flex w-64 min-w-[16rem] flex-col h-full">
        {/* Logo + collapse toggle */}
        <div className="flex h-11 items-center justify-between px-3 border-b border-border shrink-0">
          <LocksoLogo size="sm" />
          <button
            onClick={toggle}
            className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
            title={t("sidebar.collapse")}
          >
            <PanelLeftClose className="h-4 w-4" />
          </button>
        </div>

        {/* Search trigger */}
        <div className="px-2 pt-2 pb-1">
          <button
            onClick={openCommandPalette}
            className="flex w-full items-center gap-2 rounded-md border border-border bg-muted/30 px-2.5 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <Search className="h-3.5 w-3.5" />
            <span className="flex-1 text-left">{t("search.placeholder")}</span>
            <kbd className="hidden sm:inline-flex h-4 items-center rounded border border-border bg-background px-1 text-[9px] font-medium text-muted-foreground">
              ⌘K
            </kbd>
          </button>
        </div>

        {/* Quick links */}
        <div className="px-2 pt-1 pb-0.5 space-y-px">
          <SidebarLink
            to="/recent"
            icon={Clock}
            label={t("nav.recent")}
            active={isActive("/recent")}
          />
          <SidebarLink
            to="/favorites"
            icon={Star}
            label={t("nav.favorites")}
            active={isActive("/favorites")}
          />
          <SidebarLink
            to="/health"
            icon={ShieldCheck}
            label={t("nav.health")}
            active={isActive("/health")}
          />
          <SidebarLink
            to="/sends"
            icon={Send}
            label={t("nav.sends")}
            active={isActive("/sends")}
          />
          <SidebarLink
            to="/trash"
            icon={Trash2}
            label={t("nav.trash")}
            active={isActive("/trash")}
            badge={trashCount > 0 ? trashCount : undefined}
          />
        </div>

        <div className="mx-2 my-1 border-t border-border" />

        {/* Vault groups */}
        <nav className="flex-1 overflow-y-auto px-2 pb-2 space-y-px">
          {groups.map((group) => (
            <VaultGroupSection
              key={group.code}
              group={group}
              label={groupLabel(group.code)}
              collapsed={collapsedGroups.has(group.code)}
              onToggle={() => toggleGroup(group.code)}
              onVaultCreated={loadVaults}
            />
          ))}

          {groups.length === 0 && (
            <div className="px-2 py-6 text-center">
              <Lock className="mx-auto h-5 w-5 text-muted-foreground/40" />
              <p className="mt-2 text-xs text-muted-foreground">
                {t("dashboard.emptyTitle")}
              </p>
            </div>
          )}
        </nav>

        <div className="mx-2 border-t border-border" />

        {/* Bottom nav */}
        <div className="px-2 py-1.5 space-y-px shrink-0">
          <SidebarLink
            to="/settings"
            icon={Settings}
            label={t("nav.settings")}
            active={location.pathname.startsWith("/settings")}
          />
          <button
            onClick={handleLogout}
            className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <LogOut className="h-4 w-4 shrink-0" />
            <span className="text-xs">{t("nav.logout")}</span>
          </button>
        </div>

        {/* User + controls */}
        <div className="border-t border-border px-2 py-2 shrink-0">
          <div className="flex items-center gap-2">
            <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary/10">
              <Shield className="h-3 w-3 text-primary" />
            </div>
            <div className="min-w-0 flex-1">
              <p className="truncate text-xs font-medium text-foreground leading-tight">
                {user?.fullName || user?.login || "User"}
              </p>
              <p className="truncate text-[10px] text-muted-foreground leading-tight">
                {user?.email || ""}
              </p>
            </div>
          </div>
          <div className="mt-1.5 flex items-center gap-1.5">
            <LanguageToggle />
            <ThemeToggle />
          </div>
        </div>
      </div>
    </aside>
  );
}

// ─── Sidebar Link ───

function SidebarLink({
  to,
  icon: Icon,
  label,
  active,
  badge,
}: {
  to: string;
  icon: typeof Clock;
  label: string;
  active: boolean;
  badge?: number;
}) {
  return (
    <Link
      to={to}
      className={cn(
        "flex items-center gap-2 rounded-md px-2 py-1.5 text-xs transition-colors",
        active
          ? "bg-primary/10 text-primary font-medium"
          : "text-muted-foreground hover:bg-muted hover:text-foreground",
      )}
    >
      <Icon className="h-4 w-4 shrink-0" />
      <span className="flex-1">{label}</span>
      {badge !== undefined && (
        <span className="ml-auto flex h-4 min-w-[16px] items-center justify-center rounded-full bg-muted-foreground/15 px-1 text-[9px] font-medium tabular-nums">
          {badge}
        </span>
      )}
    </Link>
  );
}

// ─── Vault Group Section ───

interface VaultGroupSectionProps {
  group: VaultTypeGroup;
  label: string;
  collapsed: boolean;
  onToggle: () => void;
  onVaultCreated: () => void;
}

function VaultGroupSection({
  group,
  label,
  collapsed,
  onToggle,
  onVaultCreated,
}: VaultGroupSectionProps) {
  const { t } = useTranslation();
  const [createOpen, setCreateOpen] = useState(false);

  const handleCreateClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setCreateOpen(true);
  };

  return (
    <div className="pt-1">
      {/* Section header */}
      <div className="group flex items-center px-1">
        <button
          onClick={onToggle}
          className="flex min-w-0 flex-1 items-center gap-1 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/70 hover:text-muted-foreground transition-colors"
        >
          {collapsed ? (
            <ChevronRight className="h-3 w-3 shrink-0" />
          ) : (
            <ChevronDown className="h-3 w-3 shrink-0" />
          )}
          <span className="truncate">{label}</span>
          <span className="text-[9px] font-normal ml-1 opacity-60">
            {group.vaults.length}
          </span>
        </button>
        <button
          onClick={handleCreateClick}
          className="shrink-0 rounded p-0.5 text-muted-foreground/50 opacity-0 transition-opacity group-hover:opacity-100 hover:text-foreground hover:bg-muted"
          title={t("vault.create")}
        >
          <Plus className="h-3 w-3" />
        </button>
      </div>

      {/* Vault list */}
      {!collapsed && (
        <div className="space-y-px">
          {group.vaults.map((vault) => (
            <VaultNode key={vault.id} vault={vault} />
          ))}
        </div>
      )}

      <CreateVaultDialog
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        onCreated={() => {
          setCreateOpen(false);
          onVaultCreated();
        }}
        initialType={group.code}
      />
    </div>
  );
}

// ─── Vault Node ───

function VaultNode({ vault }: { vault: VaultListItem }) {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const [expanded, setExpanded] = useState(false);
  const [folders, setFolders] = useState<FolderTreeNode[]>([]);
  const [loaded, setLoaded] = useState(false);

  // Folder dialog state
  const [folderDialogOpen, setFolderDialogOpen] = useState(false);
  const [folderDialogMode, setFolderDialogMode] = useState<"create" | "rename">("create");
  const [folderDialogParentId, setFolderDialogParentId] = useState<string | undefined>();
  const [folderDialogFolderId, setFolderDialogFolderId] = useState<string | undefined>();
  const [folderDialogInitialName, setFolderDialogInitialName] = useState<string | undefined>();

  // Delete folder state
  const [deletingFolder, setDeletingFolder] = useState<FolderTreeNode | null>(null);
  const [deletingFolderLoading, setDeletingFolderLoading] = useState(false);

  const isInVault = location.pathname === `/vault/${vault.id}`;
  const color = getColor(vault.colorCode);
  const selectedFolderId = isInVault
    ? new URLSearchParams(location.search).get("folder")
    : null;

  useEffect(() => {
    if (isInVault && !expanded) setExpanded(true);
  }, [isInVault]);

  const reloadFolders = useCallback(async () => {
    try {
      const data = await folderApi.tree(vault.id);
      setFolders(data);
    } catch {
      // keep current
    }
  }, [vault.id]);

  useEffect(() => {
    if (expanded && !loaded) {
      reloadFolders();
      setLoaded(true);
    }
  }, [expanded, loaded, reloadFolders]);

  useEffect(() => {
    if (!loaded) return;
    const handler = () => reloadFolders();
    window.addEventListener(RELOAD_EVENT, handler);
    return () => window.removeEventListener(RELOAD_EVENT, handler);
  }, [loaded, reloadFolders]);

  const handleClick = () => {
    navigate(`/vault/${vault.id}`);
    if (!expanded) setExpanded(true);
  };

  const handleSelectFolder = (folderId: string | null) => {
    if (folderId) {
      navigate(`/vault/${vault.id}?folder=${folderId}`);
    } else {
      navigate(`/vault/${vault.id}`);
    }
  };

  const handleCreateFolder = (parentId?: string) => {
    setFolderDialogMode("create");
    setFolderDialogParentId(parentId);
    setFolderDialogFolderId(undefined);
    setFolderDialogInitialName(undefined);
    setFolderDialogOpen(true);
  };

  const handleRenameFolder = (folder: FolderTreeNode) => {
    setFolderDialogMode("rename");
    setFolderDialogFolderId(folder.id);
    setFolderDialogInitialName(folder.name);
    setFolderDialogParentId(undefined);
    setFolderDialogOpen(true);
  };

  const handleDeleteFolder = (folder: FolderTreeNode) => {
    setDeletingFolder(folder);
  };

  const confirmDeleteFolder = async () => {
    if (!deletingFolder) return;
    setDeletingFolderLoading(true);
    try {
      await folderApi.delete(deletingFolder.id);
      if (selectedFolderId === deletingFolder.id) {
        navigate(`/vault/${vault.id}`);
      }
      setDeletingFolder(null);
      await reloadFolders();
    } catch {
      // silently fail
    } finally {
      setDeletingFolderLoading(false);
    }
  };

  const handleFolderDialogSuccess = () => {
    reloadFolders();
  };

  return (
    <div>
      <div
        className={cn(
          "group flex items-center rounded-md transition-colors cursor-pointer",
          isInVault
            ? "bg-primary/8 text-foreground"
            : "text-muted-foreground hover:bg-muted hover:text-foreground",
        )}
      >
        <button
          onClick={(e) => {
            e.stopPropagation();
            setExpanded(!expanded);
          }}
          className="shrink-0 p-1 pl-1.5"
        >
          {expanded ? (
            <ChevronDown className="h-3 w-3" />
          ) : (
            <ChevronRight className="h-3 w-3" />
          )}
        </button>

        <button
          onClick={handleClick}
          className="flex min-w-0 flex-1 items-center gap-1.5 py-1.5 pr-1"
        >
          <span
            className={cn(
              "h-2 w-2 shrink-0 rounded-full",
              color.bg.replace("/15", ""),
            )}
          />
          <span
            className={cn(
              "truncate text-xs",
              isInVault && "font-medium",
            )}
          >
            {vault.name}
          </span>
          {vault.itemCount > 0 && (
            <span className="ml-auto text-[9px] text-muted-foreground/60 tabular-nums">
              {vault.itemCount}
            </span>
          )}
        </button>

        <button
          onClick={(e) => {
            e.stopPropagation();
            if (!expanded) setExpanded(true);
            handleCreateFolder();
          }}
          className="shrink-0 rounded p-0.5 mr-0.5 text-muted-foreground/50 opacity-0 transition-opacity group-hover:opacity-100 hover:text-foreground hover:bg-muted"
          title={t("folder.createTitle")}
        >
          <FolderPlus className="h-3 w-3" />
        </button>
      </div>

      {/* Inline folder tree */}
      {expanded && (
        <div className="ml-4 border-l border-border/40 pl-1">
          {folders.length > 0 && (
            <button
              onClick={() => handleSelectFolder(null)}
              className={cn(
                "flex w-full items-center gap-1.5 rounded-md py-1 px-1 text-xs transition-colors",
                isInVault && !selectedFolderId
                  ? "bg-primary/10 text-primary font-medium"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground",
              )}
            >
              <Folder className="h-3 w-3 shrink-0 text-muted-foreground/60" />
              <span className="truncate text-[11px]">{t("folder.allItems")}</span>
            </button>
          )}

          {folders.map((folder) => (
            <SidebarFolder
              key={folder.id}
              folder={folder}
              vaultId={vault.id}
              depth={0}
              selectedFolderId={selectedFolderId}
              onSelectFolder={handleSelectFolder}
              onCreateFolder={handleCreateFolder}
              onRenameFolder={handleRenameFolder}
              onDeleteFolder={handleDeleteFolder}
            />
          ))}
        </div>
      )}

      <FolderDialog
        open={folderDialogOpen}
        mode={folderDialogMode}
        vaultId={vault.id}
        parentFolderId={folderDialogParentId}
        folderId={folderDialogFolderId}
        initialName={folderDialogInitialName}
        onClose={() => setFolderDialogOpen(false)}
        onSuccess={handleFolderDialogSuccess}
      />

      {deletingFolder && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div className="w-full max-w-sm rounded-lg border border-border bg-card p-6 shadow-lg">
            <h3 className="text-lg font-semibold">{t("folder.delete")}</h3>
            <p className="mt-2 text-sm text-muted-foreground">
              {t("folder.deleteConfirm", { name: deletingFolder.name })}
            </p>
            <div className="mt-4 flex justify-end gap-3">
              <button
                onClick={() => setDeletingFolder(null)}
                disabled={deletingFolderLoading}
                className="rounded-md border border-border px-3 py-1.5 text-sm font-medium text-foreground hover:bg-muted disabled:opacity-50"
              >
                {t("vault.cancel")}
              </button>
              <button
                onClick={confirmDeleteFolder}
                disabled={deletingFolderLoading}
                className="rounded-md bg-destructive px-3 py-1.5 text-sm font-medium text-white hover:bg-destructive/90 disabled:opacity-50"
              >
                {t("folder.delete")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Sidebar Folder Node ───

function SidebarFolder({
  folder,
  vaultId,
  depth,
  selectedFolderId,
  onSelectFolder,
  onCreateFolder,
  onRenameFolder,
  onDeleteFolder,
}: {
  folder: FolderTreeNode;
  vaultId: string;
  depth: number;
  selectedFolderId: string | null;
  onSelectFolder: (id: string | null) => void;
  onCreateFolder: (parentId?: string) => void;
  onRenameFolder: (folder: FolderTreeNode) => void;
  onDeleteFolder: (folder: FolderTreeNode) => void;
}) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const location = useLocation();
  const hasChildren = folder.children.length > 0;

  const params = new URLSearchParams(location.search);
  const isSelected =
    location.pathname === `/vault/${vaultId}` &&
    params.get("folder") === folder.id;

  return (
    <div>
      <div
        className={cn(
          "group flex items-center rounded-md text-sm transition-colors",
          isSelected
            ? "bg-primary/10 text-primary font-medium"
            : "text-muted-foreground hover:bg-muted hover:text-foreground",
        )}
        style={{ paddingLeft: `${depth * 10}px` }}
      >
        <button
          onClick={(e) => {
            e.stopPropagation();
            setExpanded(!expanded);
          }}
          className="shrink-0 p-0.5"
        >
          {hasChildren ? (
            expanded ? (
              <ChevronDown className="h-3 w-3" />
            ) : (
              <ChevronRight className="h-3 w-3" />
            )
          ) : (
            <span className="inline-block w-3" />
          )}
        </button>

        <button
          onClick={() => onSelectFolder(folder.id)}
          className="flex min-w-0 flex-1 items-center gap-1.5 py-1 pr-1"
        >
          {expanded && hasChildren ? (
            <FolderOpen className="h-3 w-3 shrink-0 text-muted-foreground/60" />
          ) : (
            <Folder className="h-3 w-3 shrink-0 text-muted-foreground/60" />
          )}
          <span className="truncate text-[11px]">{folder.name}</span>
          {folder.itemCount > 0 && (
            <span className="ml-auto text-[9px] text-muted-foreground/50 tabular-nums">
              {folder.itemCount}
            </span>
          )}
        </button>

        {/* Context menu trigger */}
        <div className="relative shrink-0">
          <button
            onClick={(e) => {
              e.stopPropagation();
              setMenuOpen(!menuOpen);
            }}
            className="rounded p-0.5 text-muted-foreground opacity-0 transition-opacity hover:bg-muted group-hover:opacity-100"
          >
            <MoreVertical className="h-3 w-3" />
          </button>

          {menuOpen && (
            <>
              <div className="fixed inset-0 z-10" onClick={() => setMenuOpen(false)} />
              <div className="absolute right-0 top-5 z-20 min-w-[140px] rounded-md border border-border bg-card py-1 shadow-lg">
                <button
                  onClick={() => {
                    setMenuOpen(false);
                    onCreateFolder(folder.id);
                  }}
                  className="flex w-full items-center gap-2 px-3 py-1.5 text-xs text-foreground hover:bg-muted"
                >
                  <Plus className="h-3 w-3" />
                  {t("folder.createSub")}
                </button>
                <button
                  onClick={() => {
                    setMenuOpen(false);
                    onRenameFolder(folder);
                  }}
                  className="flex w-full items-center gap-2 px-3 py-1.5 text-xs text-foreground hover:bg-muted"
                >
                  <Pencil className="h-3 w-3" />
                  {t("folder.rename")}
                </button>
                <button
                  onClick={() => {
                    setMenuOpen(false);
                    onDeleteFolder(folder);
                  }}
                  className="flex w-full items-center gap-2 px-3 py-1.5 text-xs text-destructive hover:bg-muted"
                >
                  <Trash2 className="h-3 w-3" />
                  {t("folder.delete")}
                </button>
              </div>
            </>
          )}
        </div>
      </div>

      {expanded &&
        hasChildren &&
        folder.children.map((child) => (
          <SidebarFolder
            key={child.id}
            folder={child}
            vaultId={vaultId}
            depth={depth + 1}
            selectedFolderId={selectedFolderId}
            onSelectFolder={onSelectFolder}
            onCreateFolder={onCreateFolder}
            onRenameFolder={onRenameFolder}
            onDeleteFolder={onDeleteFolder}
          />
        ))}
    </div>
  );
}
