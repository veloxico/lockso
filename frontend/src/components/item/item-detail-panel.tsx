import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import {
  Eye,
  EyeOff,
  Copy,
  Check,
  ExternalLink,
  Star,
  Pencil,
  Trash2,
  History,
  Globe,
  Key,
  Tag,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Spinner } from "@/components/ui/spinner";
import { getColor } from "@/lib/colors";
import { itemApi } from "@/api/vaults";
import { AttachmentSection } from "./attachment-section";
import { PasswordAgeBadge } from "./password-age-badge";
import { TotpDisplay } from "./totp-display";
import type { ItemView } from "@/types/vault";

/** Time in ms before clipboard is auto-cleared after copy (30 seconds). */
const CLIPBOARD_CLEAR_MS = 30_000;

interface ItemDetailPanelProps {
  itemId: string | null;
  onEdit: (item: ItemView) => void;
  onDelete: (item: ItemView) => void;
  onViewSnapshots: (item: ItemView) => void;
  onFavoriteToggled: () => void;
}

export function ItemDetailPanel({
  itemId,
  onEdit,
  onDelete,
  onViewSnapshots,
  onFavoriteToggled,
}: ItemDetailPanelProps) {
  const { t } = useTranslation();
  const [item, setItem] = useState<ItemView | null>(null);
  const [loading, setLoading] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  const loadItem = useCallback(async () => {
    if (!itemId) {
      setItem(null);
      return;
    }

    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;

    setLoading(true);
    setShowPassword(false);
    try {
      const data = await itemApi.get(itemId);
      if (!controller.signal.aborted) {
        setItem(data);
      }
    } catch {
      if (!controller.signal.aborted) {
        setItem(null);
      }
    } finally {
      if (!controller.signal.aborted) {
        setLoading(false);
      }
    }
  }, [itemId]);

  useEffect(() => {
    loadItem();
    return () => {
      abortRef.current?.abort();
    };
  }, [loadItem]);

  const copyTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const clipboardClearRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    return () => {
      clearTimeout(copyTimerRef.current);
      clearTimeout(clipboardClearRef.current);
    };
  }, []);

  const copyToClipboard = async (text: string, field: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedField(field);
      clearTimeout(copyTimerRef.current);
      copyTimerRef.current = setTimeout(() => setCopiedField(null), 2000);

      clearTimeout(clipboardClearRef.current);
      clipboardClearRef.current = setTimeout(() => {
        navigator.clipboard.writeText("").catch(() => {});
      }, CLIPBOARD_CLEAR_MS);
    } catch {
      // Clipboard API not available
    }
  };

  const handleToggleFavorite = async () => {
    if (!item) return;
    try {
      const result = await itemApi.toggleFavorite(item.id);
      setItem({ ...item, isFavorite: result.isFavorite });
      onFavoriteToggled();
    } catch {
      // Ignore
    }
  };

  if (!itemId) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        <div className="text-center">
          <Key className="mx-auto h-8 w-8 opacity-30" />
          <p className="mt-2 text-xs">{t("item.selectHint")}</p>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Spinner size="lg" />
      </div>
    );
  }

  if (!item) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        <p className="text-sm">{t("item.notFound")}</p>
      </div>
    );
  }

  const color = getColor(item.colorCode);
  const visibleCustoms = item.customs.filter((c) => !c.name.startsWith("_"));

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center gap-3 border-b border-border px-5 py-3 shrink-0">
        <div className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-lg ${color.bg}`}>
          {item.url ? (
            <Globe className={`h-4 w-4 ${color.text}`} />
          ) : (
            <Key className={`h-4 w-4 ${color.text}`} />
          )}
        </div>

        <div className="min-w-0 flex-1">
          <h2 className="text-sm font-semibold text-foreground truncate">{item.name}</h2>
          {item.url && (
            /^https?:\/\//i.test(item.url) ? (
              <a
                href={item.url}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1 text-xs text-primary hover:underline truncate"
              >
                {item.url}
                <ExternalLink className="h-2.5 w-2.5 shrink-0" />
              </a>
            ) : (
              <span className="text-xs text-muted-foreground truncate block">{item.url}</span>
            )
          )}
        </div>

        <div className="flex shrink-0 gap-0.5">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={handleToggleFavorite}
            title={t(item.isFavorite ? "item.unfavorite" : "item.favorite")}
          >
            <Star
              className={`h-3.5 w-3.5 ${item.isFavorite ? "fill-amber-400 text-amber-400" : ""}`}
            />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => onEdit(item)}
            title={t("item.edit")}
          >
            <Pencil className="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => onViewSnapshots(item)}
            title={t("item.history")}
          >
            <History className="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => onDelete(item)}
            title={t("item.delete")}
          >
            <Trash2 className="h-3.5 w-3.5 text-destructive" />
          </Button>
        </div>
      </div>

      {/* Fields */}
      <div className="flex-1 overflow-y-auto px-5 py-4 space-y-4">
        {/* Login */}
        {item.login && (
          <FieldRow
            label={t("item.loginLabel")}
            value={item.login}
            onCopy={() => copyToClipboard(item.login, "login")}
            copied={copiedField === "login"}
          />
        )}

        {/* Password */}
        {item.password && (
          <div>
            <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider mb-1">
              {t("item.passwordLabel")}
            </p>
            <div className="flex items-center gap-1.5">
              <code className="flex-1 rounded bg-muted px-2.5 py-1.5 text-xs font-mono truncate">
                {showPassword ? item.password : "••••••••••••"}
              </code>
              <IconBtn
                onClick={() => setShowPassword(!showPassword)}
                title={t(showPassword ? "login.hidePassword" : "login.showPassword")}
              >
                {showPassword ? <EyeOff className="h-3.5 w-3.5" /> : <Eye className="h-3.5 w-3.5" />}
              </IconBtn>
              <CopyBtn
                onClick={() => copyToClipboard(item.password, "password")}
                copied={copiedField === "password"}
              />
            </div>
            <PasswordAgeBadge passwordChangedAt={item.passwordChangedAt} />
          </div>
        )}

        {/* URL */}
        {item.url && (
          <FieldRow
            label={t("item.urlLabel")}
            value={item.url}
            onCopy={() => copyToClipboard(item.url, "url")}
            copied={copiedField === "url"}
          />
        )}

        {/* Description */}
        {item.description && (
          <div>
            <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider mb-1">
              {t("item.descriptionLabel")}
            </p>
            <p className="text-xs text-foreground whitespace-pre-wrap">{item.description}</p>
          </div>
        )}

        {/* Custom fields */}
        {visibleCustoms.length > 0 && (
          <div className="space-y-2">
            <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
              {t("item.customFields")}
            </p>
            {visibleCustoms.map((field, i) =>
              field.type === "totp" && field.value ? (
                <TotpDisplay
                  key={i}
                  name={field.name}
                  secret={field.value}
                />
              ) : (
                <CustomFieldRow
                  key={i}
                  name={field.name}
                  value={field.value}
                  type={field.type}
                  onCopy={() => copyToClipboard(field.value, `custom-${i}`)}
                  copied={copiedField === `custom-${i}`}
                />
              ),
            )}
          </div>
        )}

        {/* Tags */}
        {item.tags.length > 0 && (
          <div>
            <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider mb-1">
              {t("item.tagsLabel")}
            </p>
            <div className="flex flex-wrap gap-1">
              {item.tags.map((tag) => (
                <Badge key={tag} variant="secondary" className="text-[10px] py-0 px-1.5">
                  <Tag className="mr-0.5 h-2.5 w-2.5" />
                  {tag}
                </Badge>
              ))}
            </div>
          </div>
        )}

        {/* Attachments */}
        <AttachmentSection itemId={item.id} />

        {/* Metadata */}
        <div className="border-t border-border pt-3 text-[10px] text-muted-foreground space-y-0.5">
          <p>
            {t("item.createdAt")}: {new Date(item.createdAt).toLocaleString()}
          </p>
          <p>
            {t("item.updatedAt")}: {new Date(item.updatedAt).toLocaleString()}
          </p>
        </div>
      </div>
    </div>
  );
}

function FieldRow({
  label,
  value,
  onCopy,
  copied,
}: {
  label: string;
  value: string;
  onCopy: () => void;
  copied: boolean;
}) {
  return (
    <div>
      <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider mb-1">
        {label}
      </p>
      <div className="flex items-center gap-1.5">
        <span className="flex-1 truncate text-xs text-foreground">{value}</span>
        <CopyBtn onClick={onCopy} copied={copied} />
      </div>
    </div>
  );
}

function IconBtn({
  onClick,
  title,
  children,
}: {
  onClick: () => void;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      className="shrink-0 rounded-md p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
    >
      {children}
    </button>
  );
}

function CopyBtn({ onClick, copied }: { onClick: () => void; copied: boolean }) {
  const { t } = useTranslation();
  return (
    <button
      onClick={onClick}
      title={t("item.copy")}
      className="shrink-0 rounded-md p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
    >
      {copied ? (
        <Check className="h-3.5 w-3.5 text-emerald-500" />
      ) : (
        <Copy className="h-3.5 w-3.5" />
      )}
    </button>
  );
}

function CustomFieldRow({
  name,
  value,
  type,
  onCopy,
  copied,
}: {
  name: string;
  value: string;
  type: string;
  onCopy: () => void;
  copied: boolean;
}) {
  const [show, setShow] = useState(false);
  const isSecret = type === "password" || type === "totp";

  return (
    <div className="rounded-md border border-border p-2.5">
      <p className="text-[10px] text-muted-foreground mb-1">{name}</p>
      <div className="flex items-center gap-1.5">
        <span className="flex-1 truncate text-xs font-mono">
          {isSecret && !show ? "••••••••" : value}
        </span>
        {isSecret && (
          <IconBtn onClick={() => setShow(!show)} title={show ? "Hide" : "Show"}>
            {show ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
          </IconBtn>
        )}
        <CopyBtn onClick={onCopy} copied={copied} />
      </div>
    </div>
  );
}
