import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  ChevronRight,
  ChevronDown,
  Folder,
  FolderOpen,
  Plus,
  MoreVertical,
  Pencil,
  Trash2,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { FolderTreeNode } from "@/types/vault";

interface FolderTreeProps {
  folders: FolderTreeNode[];
  selectedFolderId: string | null;
  onSelectFolder: (id: string | null) => void;
  onCreateFolder: (parentId?: string) => void;
  onRenameFolder: (folder: FolderTreeNode) => void;
  onDeleteFolder: (folder: FolderTreeNode) => void;
}

export function FolderTree({
  folders,
  selectedFolderId,
  onSelectFolder,
  onCreateFolder,
  onRenameFolder,
  onDeleteFolder,
}: FolderTreeProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-0.5">
      {/* All items (root) */}
      <button
        onClick={() => onSelectFolder(null)}
        className={cn(
          "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors",
          selectedFolderId === null
            ? "bg-primary/10 text-primary font-medium"
            : "text-muted-foreground hover:bg-muted hover:text-foreground",
        )}
      >
        <Folder className="h-4 w-4 shrink-0" />
        <span className="truncate">{t("folder.allItems")}</span>
      </button>

      {/* Folder nodes */}
      {folders.map((folder) => (
        <FolderNode
          key={folder.id}
          folder={folder}
          depth={0}
          selectedFolderId={selectedFolderId}
          onSelectFolder={onSelectFolder}
          onCreateFolder={onCreateFolder}
          onRenameFolder={onRenameFolder}
          onDeleteFolder={onDeleteFolder}
        />
      ))}

      {/* Create folder button */}
      <button
        onClick={() => onCreateFolder()}
        className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
      >
        <Plus className="h-4 w-4 shrink-0" />
        <span>{t("folder.create")}</span>
      </button>
    </div>
  );
}

interface FolderNodeProps {
  folder: FolderTreeNode;
  depth: number;
  selectedFolderId: string | null;
  onSelectFolder: (id: string | null) => void;
  onCreateFolder: (parentId?: string) => void;
  onRenameFolder: (folder: FolderTreeNode) => void;
  onDeleteFolder: (folder: FolderTreeNode) => void;
}

function FolderNode({
  folder,
  depth,
  selectedFolderId,
  onSelectFolder,
  onCreateFolder,
  onRenameFolder,
  onDeleteFolder,
}: FolderNodeProps) {
  const [expanded, setExpanded] = useState(true);
  const [menuOpen, setMenuOpen] = useState(false);
  const hasChildren = folder.children.length > 0;
  const isSelected = selectedFolderId === folder.id;
  const Icon = expanded && hasChildren ? FolderOpen : Folder;

  return (
    <div>
      <div
        className={cn(
          "group relative flex items-center rounded-md text-sm transition-colors",
          isSelected
            ? "bg-primary/10 text-primary font-medium"
            : "text-muted-foreground hover:bg-muted hover:text-foreground",
        )}
        style={{ paddingLeft: `${(depth + 1) * 12}px` }}
      >
        {/* Expand/collapse */}
        <button
          onClick={(e) => {
            e.stopPropagation();
            setExpanded(!expanded);
          }}
          className="shrink-0 p-0.5"
        >
          {hasChildren ? (
            expanded ? (
              <ChevronDown className="h-3.5 w-3.5" />
            ) : (
              <ChevronRight className="h-3.5 w-3.5" />
            )
          ) : (
            <span className="inline-block w-3.5" />
          )}
        </button>

        {/* Folder name */}
        <button
          onClick={() => onSelectFolder(folder.id)}
          className="flex min-w-0 flex-1 items-center gap-1.5 py-1.5 pr-2"
        >
          <Icon className="h-4 w-4 shrink-0" />
          <span className="truncate">{folder.name}</span>
          {folder.itemCount > 0 && (
            <span className="ml-auto text-xs text-muted-foreground">
              {folder.itemCount}
            </span>
          )}
        </button>

        {/* Context menu */}
        <div className="relative shrink-0">
          <button
            onClick={(e) => {
              e.stopPropagation();
              setMenuOpen(!menuOpen);
            }}
            className="rounded p-0.5 text-muted-foreground opacity-0 transition-opacity hover:bg-muted group-hover:opacity-100"
          >
            <MoreVertical className="h-3.5 w-3.5" />
          </button>

          {menuOpen && (
            <FolderContextMenu
              onClose={() => setMenuOpen(false)}
              onCreateSub={() => {
                setMenuOpen(false);
                onCreateFolder(folder.id);
              }}
              onRename={() => {
                setMenuOpen(false);
                onRenameFolder(folder);
              }}
              onDelete={() => {
                setMenuOpen(false);
                onDeleteFolder(folder);
              }}
            />
          )}
        </div>
      </div>

      {/* Children */}
      {expanded &&
        hasChildren &&
        folder.children.map((child) => (
          <FolderNode
            key={child.id}
            folder={child}
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

interface FolderContextMenuProps {
  onClose: () => void;
  onCreateSub: () => void;
  onRename: () => void;
  onDelete: () => void;
}

function FolderContextMenu({
  onClose,
  onCreateSub,
  onRename,
  onDelete,
}: FolderContextMenuProps) {
  const { t } = useTranslation();

  return (
    <>
      <div className="fixed inset-0 z-10" onClick={onClose} />
      <div className="absolute right-0 top-6 z-20 min-w-[160px] rounded-md border border-border bg-card py-1 shadow-lg">
        <button
          onClick={onCreateSub}
          className="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-foreground hover:bg-muted"
        >
          <Plus className="h-3.5 w-3.5" />
          {t("folder.createSub")}
        </button>
        <button
          onClick={onRename}
          className="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-foreground hover:bg-muted"
        >
          <Pencil className="h-3.5 w-3.5" />
          {t("folder.rename")}
        </button>
        <button
          onClick={onDelete}
          className="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-destructive hover:bg-muted"
        >
          <Trash2 className="h-3.5 w-3.5" />
          {t("folder.delete")}
        </button>
      </div>
    </>
  );
}
