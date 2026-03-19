import { Lock, Folder, MoreVertical, Pencil, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useState, useRef, useEffect } from "react";
import { getColor } from "@/lib/colors";
import type { VaultListItem } from "@/types/vault";

interface VaultCardProps {
  vault: VaultListItem;
  onClick: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

export function VaultCard({ vault, onClick, onEdit, onDelete }: VaultCardProps) {
  const { t } = useTranslation();
  const color = getColor(vault.colorCode);
  const [menuOpen, setMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!menuOpen) return;
    const handler = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [menuOpen]);

  return (
    <div
      onClick={onClick}
      className="group relative flex cursor-pointer flex-col rounded-lg border border-border bg-card p-5 transition-colors hover:border-primary/30 hover:bg-card/80"
    >
      {/* Menu button */}
      <div className="absolute right-3 top-3" ref={menuRef}>
        <button
          onClick={(e) => {
            e.stopPropagation();
            setMenuOpen(!menuOpen);
          }}
          className="rounded-md p-1 text-muted-foreground opacity-0 transition-opacity hover:bg-muted group-hover:opacity-100"
        >
          <MoreVertical className="h-4 w-4" />
        </button>

        {menuOpen && (
          <div className="absolute right-0 top-8 z-10 min-w-[140px] rounded-md border border-border bg-card py-1 shadow-lg">
            <button
              onClick={(e) => {
                e.stopPropagation();
                setMenuOpen(false);
                onEdit();
              }}
              className="flex w-full items-center gap-2 px-3 py-2 text-sm text-foreground hover:bg-muted"
            >
              <Pencil className="h-3.5 w-3.5" />
              {t("vault.edit")}
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                setMenuOpen(false);
                onDelete();
              }}
              className="flex w-full items-center gap-2 px-3 py-2 text-sm text-destructive hover:bg-muted"
            >
              <Trash2 className="h-3.5 w-3.5" />
              {t("vault.delete")}
            </button>
          </div>
        )}
      </div>

      {/* Icon */}
      <div className={`flex h-10 w-10 items-center justify-center rounded-lg ${color.bg}`}>
        <Lock className={`h-5 w-5 ${color.text}`} />
      </div>

      {/* Info */}
      <h3 className="mt-3 font-semibold text-foreground truncate">{vault.name}</h3>
      {vault.description && (
        <p className="mt-1 text-sm text-muted-foreground line-clamp-2">
          {vault.description}
        </p>
      )}

      {/* Stats */}
      <div className="mt-auto flex items-center gap-4 pt-4 text-xs text-muted-foreground">
        <span className="flex items-center gap-1">
          <Lock className="h-3 w-3" />
          {t("vault.itemCount", { count: vault.itemCount })}
        </span>
        <span className="flex items-center gap-1">
          <Folder className="h-3 w-3" />
          {t("vault.folderCount", { count: vault.folderCount })}
        </span>
      </div>
    </div>
  );
}
