import { useTranslation } from "react-i18next";
import { Key, Globe, Star } from "lucide-react";
import { cn } from "@/lib/utils";
import { getColor } from "@/lib/colors";
import type { ItemListEntry } from "@/types/vault";

interface ItemListProps {
  items: ItemListEntry[];
  selectedItemId: string | null;
  onSelectItem: (id: string) => void;
  loading?: boolean;
}

export function ItemList({ items, selectedItemId, onSelectItem, loading }: ItemListProps) {
  const { t } = useTranslation();

  if (loading) {
    return (
      <div className="flex flex-col gap-2 p-2">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="h-16 animate-pulse rounded-md bg-muted" />
        ))}
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12 text-center">
        <Key className="h-8 w-8 text-muted-foreground" />
        <p className="mt-3 text-sm text-muted-foreground">{t("item.emptyList")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-0.5">
      {items.map((item) => (
        <ItemRow
          key={item.id}
          item={item}
          isSelected={selectedItemId === item.id}
          onSelect={() => onSelectItem(item.id)}
        />
      ))}
    </div>
  );
}

interface ItemRowProps {
  item: ItemListEntry;
  isSelected: boolean;
  onSelect: () => void;
}

function ItemRow({ item, isSelected, onSelect }: ItemRowProps) {
  const color = getColor(item.colorCode);

  return (
    <button
      onClick={onSelect}
      className={cn(
        "flex items-center gap-3 rounded-md px-3 py-2.5 text-left transition-colors",
        isSelected
          ? "bg-primary/10 border border-primary/20"
          : "hover:bg-muted border border-transparent",
      )}
    >
      {/* Icon */}
      <div className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-md ${color.bg}`}>
        {item.url ? (
          <Globe className={`h-4 w-4 ${color.text}`} />
        ) : (
          <Key className={`h-4 w-4 ${color.text}`} />
        )}
      </div>

      {/* Content */}
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="truncate text-sm font-medium text-foreground">
            {item.name}
          </span>
          {item.isFavorite && (
            <Star className="h-3 w-3 shrink-0 fill-amber-400 text-amber-400" />
          )}
        </div>
        {item.login && (
          <p className="truncate text-xs text-muted-foreground">{item.login}</p>
        )}
      </div>
    </button>
  );
}
