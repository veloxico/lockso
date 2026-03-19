import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Search } from "lucide-react";
import {
  KeyRound,
  StickyNote,
  CreditCard,
  Contact,
  Lock,
  FileText,
  Terminal,
  Code,
  Database,
  Landmark,
  Wifi,
  Car,
  Bitcoin,
  BadgeCheck,
  HeartPulse,
  Globe,
  Server,
} from "lucide-react";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogContent,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { FEATURED_TYPES, EXTENDED_TYPES, type ItemTypeDef } from "@/lib/item-types";

// Map icon name string to actual Lucide component
const ICON_MAP: Record<string, typeof KeyRound> = {
  KeyRound,
  StickyNote,
  CreditCard,
  Contact,
  Lock,
  FileText,
  Terminal,
  Code,
  Database,
  Landmark,
  Wifi,
  Car,
  Bitcoin,
  BadgeCheck,
  HeartPulse,
  Globe,
  Server,
};

interface ItemTypePickerProps {
  open: boolean;
  onClose: () => void;
  onSelect: (type: ItemTypeDef) => void;
}

export function ItemTypePicker({ open, onClose, onSelect }: ItemTypePickerProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");

  const allTypes = [...FEATURED_TYPES, ...EXTENDED_TYPES];
  const query = search.toLowerCase().trim();

  const filtered = query
    ? allTypes.filter((typ) => t(typ.labelKey).toLowerCase().includes(query))
    : null;

  const handleSelect = (typ: ItemTypeDef) => {
    setSearch("");
    onSelect(typ);
  };

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogHeader>
        <DialogTitle>{t("itemType.pickerTitle")}</DialogTitle>
      </DialogHeader>
      <DialogContent>
        <div className="max-h-[65vh] overflow-y-auto space-y-4 pr-1">
          {/* Search */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder={t("itemType.searchPlaceholder")}
              className="pl-9"
              autoFocus
            />
          </div>

          {filtered ? (
            /* Search results */
            <div className="space-y-1">
              {filtered.length === 0 && (
                <p className="py-4 text-center text-sm text-muted-foreground">
                  {t("search.noResults")}
                </p>
              )}
              {filtered.map((typ) => (
                <TypeListItem key={typ.code} type={typ} onClick={() => handleSelect(typ)} />
              ))}
            </div>
          ) : (
            <>
              {/* Featured grid */}
              <div className="grid grid-cols-3 gap-2">
                {FEATURED_TYPES.map((typ) => (
                  <TypeCard key={typ.code} type={typ} onClick={() => handleSelect(typ)} />
                ))}
              </div>

              <div className="border-t border-border" />

              {/* Extended list */}
              <div className="space-y-1">
                {EXTENDED_TYPES.map((typ) => (
                  <TypeListItem key={typ.code} type={typ} onClick={() => handleSelect(typ)} />
                ))}
              </div>
            </>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}

// ── Featured type card (top grid) ──

function TypeCard({ type, onClick }: { type: ItemTypeDef; onClick: () => void }) {
  const { t } = useTranslation();
  const Icon = ICON_MAP[type.icon] || KeyRound;

  return (
    <button
      onClick={onClick}
      className="flex flex-col items-start gap-2.5 rounded-lg border border-border p-3 text-left transition-colors hover:bg-muted hover:border-muted-foreground/20"
    >
      <div className={cn("flex h-10 w-10 items-center justify-center rounded-lg", type.iconBg)}>
        <Icon className={cn("h-5 w-5", type.iconColor)} />
      </div>
      <span className="text-sm font-medium text-foreground leading-tight">
        {t(type.labelKey)}
      </span>
    </button>
  );
}

// ── Extended type list item ──

function TypeListItem({ type, onClick }: { type: ItemTypeDef; onClick: () => void }) {
  const { t } = useTranslation();
  const Icon = ICON_MAP[type.icon] || KeyRound;

  return (
    <button
      onClick={onClick}
      className="flex w-full items-center gap-3 rounded-lg border border-border px-3 py-2.5 text-left transition-colors hover:bg-muted hover:border-muted-foreground/20"
    >
      <div className={cn("flex h-9 w-9 shrink-0 items-center justify-center rounded-lg", type.iconBg)}>
        <Icon className={cn("h-4.5 w-4.5", type.iconColor)} />
      </div>
      <span className="text-sm font-medium text-foreground">{t(type.labelKey)}</span>
    </button>
  );
}
