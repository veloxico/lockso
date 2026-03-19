import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Send,
  Plus,
  Trash2,
  Copy,
  Check,
  ExternalLink,
  Lock,
  Clock,
  Eye,
} from "lucide-react";
import { AppLayout } from "@/components/layout/app-layout";
import { Spinner } from "@/components/ui/spinner";
import { sendApi } from "@/api/vaults";
import {
  generateSendKey,
  encryptSendPayload,
} from "@/lib/send-crypto";
import type { SendListEntry } from "@/types/vault";

export function SendsPage() {
  const { t } = useTranslation();
  const [sends, setSends] = useState<SendListEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [showDialog, setShowDialog] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  // Created link state
  const [createdLink, setCreatedLink] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const load = useCallback(async () => {
    try {
      const data = await sendApi.list();
      setSends(data);
    } catch {
      // keep current
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleDelete = async (id: string) => {
    setDeletingId(id);
    try {
      await sendApi.delete(id);
      setSends((prev) => prev.filter((s) => s.id !== id));
    } catch {
      // silent
    } finally {
      setDeletingId(null);
    }
  };

  const handleCopyLink = async (link: string) => {
    await navigator.clipboard.writeText(link);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <AppLayout>
      <div className="flex items-center justify-between gap-4">
        <h1 className="text-2xl font-bold tracking-tight shrink-0">
          {t("send.title")}
        </h1>
        <button
          onClick={() => setShowDialog(true)}
          className="flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90"
        >
          <Plus className="h-3.5 w-3.5" />
          {t("send.create")}
        </button>
      </div>

      <p className="mt-1 text-sm text-muted-foreground">
        {t("send.description")}
      </p>

      {/* Created link banner */}
      {createdLink && (
        <div className="mt-4 rounded-lg border border-emerald-500/30 bg-emerald-500/5 p-4">
          <p className="text-sm font-medium text-emerald-600 dark:text-emerald-400">
            {t("send.linkCreated")}
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            {t("send.linkWarning")}
          </p>
          <div className="mt-2 flex items-center gap-2">
            <code className="flex-1 rounded-md bg-muted px-3 py-2 text-xs font-mono text-foreground break-all">
              {createdLink}
            </code>
            <button
              onClick={() => handleCopyLink(createdLink)}
              className="shrink-0 rounded-md border border-border px-3 py-2 text-sm hover:bg-muted"
            >
              {copied ? (
                <Check className="h-4 w-4 text-emerald-500" />
              ) : (
                <Copy className="h-4 w-4" />
              )}
            </button>
          </div>
        </div>
      )}

      {loading ? (
        <div className="mt-16 flex flex-col items-center justify-center gap-3">
          <Spinner size="lg" />
        </div>
      ) : sends.length === 0 ? (
        <div className="mt-16 flex flex-col items-center justify-center text-center">
          <div className="flex h-16 w-16 items-center justify-center rounded-full bg-muted">
            <Send className="h-8 w-8 text-muted-foreground" />
          </div>
          <h2 className="mt-4 text-lg font-semibold">{t("send.empty")}</h2>
          <p className="mt-2 max-w-sm text-sm text-muted-foreground">
            {t("send.emptyDescription")}
          </p>
        </div>
      ) : (
        <div className="mt-4 space-y-2">
          {sends.map((send) => (
            <SendRow
              key={send.id}
              send={send}
              deleting={deletingId === send.id}
              onDelete={() => handleDelete(send.id)}
            />
          ))}
        </div>
      )}

      {showDialog && (
        <CreateSendDialog
          onClose={() => setShowDialog(false)}
          onCreated={(link) => {
            setCreatedLink(link);
            setShowDialog(false);
            load();
          }}
          creating={creating}
          setCreating={setCreating}
        />
      )}
    </AppLayout>
  );
}

function SendRow({
  send,
  deleting,
  onDelete,
}: {
  send: SendListEntry;
  deleting: boolean;
  onDelete: () => void;
}) {
  const { t } = useTranslation();

  const isActive = !send.isExpired && !send.isConsumed;
  const statusLabel = send.isExpired
    ? t("send.expired")
    : send.isConsumed
      ? t("send.consumed")
      : t("send.active");
  const statusColor = isActive
    ? "text-emerald-500 bg-emerald-500/10"
    : "text-muted-foreground bg-muted";

  return (
    <div className="flex items-center gap-3 rounded-lg border border-border bg-card p-3">
      <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
        <Send className="h-4 w-4 text-primary" />
      </div>

      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className={`inline-flex items-center rounded-full px-1.5 py-0.5 text-[10px] font-medium ${statusColor}`}>
            {statusLabel}
          </span>
          {send.hasPassphrase && (
            <Lock className="h-3 w-3 text-muted-foreground" />
          )}
        </div>
        <div className="mt-0.5 flex items-center gap-3 text-xs text-muted-foreground">
          <span className="flex items-center gap-1">
            <Eye className="h-3 w-3" />
            {send.viewCount}/{send.maxViews}
          </span>
          <span className="flex items-center gap-1">
            <Clock className="h-3 w-3" />
            {new Date(send.expiresAt).toLocaleDateString()}
          </span>
        </div>
      </div>

      <div className="flex items-center gap-1 shrink-0">
        {isActive && (
          <a
            href={`${window.location.origin}/send/${send.accessId}`}
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-md p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground"
            title={t("send.openLink")}
          >
            <ExternalLink className="h-4 w-4" />
          </a>
        )}
        <button
          onClick={onDelete}
          disabled={deleting}
          className="rounded-md p-1.5 text-muted-foreground hover:bg-destructive/10 hover:text-destructive disabled:opacity-50"
          title={t("send.delete")}
        >
          <Trash2 className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}

function CreateSendDialog({
  onClose,
  onCreated,
  creating,
  setCreating,
}: {
  onClose: () => void;
  onCreated: (link: string) => void;
  creating: boolean;
  setCreating: (v: boolean) => void;
}) {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const [passphrase, setPassphrase] = useState("");
  const [maxViews, setMaxViews] = useState(1);
  const [ttlHours, setTtlHours] = useState(24);
  const [error, setError] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!text.trim()) {
      setError(t("send.errorTextRequired"));
      return;
    }

    setCreating(true);
    setError("");

    try {
      // 1. Generate encryption key
      const key = generateSendKey();

      // 2. Encrypt payload client-side
      const ciphertextB64 = await encryptSendPayload(text, key);

      // 3. Create on server (server never sees plaintext or key)
      const result = await sendApi.create({
        ciphertextB64,
        passphrase: passphrase || undefined,
        maxViews,
        ttlHours,
      });

      // 4. Build link with key in fragment (never sent to server)
      const link = `${window.location.origin}/send/${result.accessId}#${key}`;
      onCreated(link);
    } catch {
      setError(t("send.errorCreateFailed"));
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg"
      >
        <h2 className="text-lg font-semibold">{t("send.createTitle")}</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          {t("send.createDescription")}
        </p>

        <div className="mt-4 space-y-4">
          {/* Secret text */}
          <div>
            <label className="block text-sm font-medium text-foreground">
              {t("send.textLabel")}
            </label>
            <textarea
              value={text}
              onChange={(e) => setText(e.target.value)}
              rows={4}
              className="mt-1 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary resize-none"
              placeholder={t("send.textPlaceholder")}
              autoFocus
            />
          </div>

          {/* Passphrase */}
          <div>
            <label className="block text-sm font-medium text-foreground">
              {t("send.passphraseLabel")}
            </label>
            <input
              type="text"
              value={passphrase}
              onChange={(e) => setPassphrase(e.target.value)}
              className="mt-1 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
              placeholder={t("send.passphrasePlaceholder")}
            />
            <p className="mt-0.5 text-xs text-muted-foreground">
              {t("send.passphraseHint")}
            </p>
          </div>

          {/* Max views + TTL */}
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-sm font-medium text-foreground">
                {t("send.maxViewsLabel")}
              </label>
              <input
                type="number"
                min={1}
                max={100}
                value={maxViews}
                onChange={(e) => setMaxViews(Number(e.target.value))}
                className="mt-1 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-foreground">
                {t("send.ttlLabel")}
              </label>
              <select
                value={ttlHours}
                onChange={(e) => setTtlHours(Number(e.target.value))}
                className="mt-1 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
              >
                <option value={1}>{t("send.ttl1h")}</option>
                <option value={6}>{t("send.ttl6h")}</option>
                <option value={24}>{t("send.ttl24h")}</option>
                <option value={72}>{t("send.ttl3d")}</option>
                <option value={168}>{t("send.ttl7d")}</option>
                <option value={720}>{t("send.ttl30d")}</option>
              </select>
            </div>
          </div>

          {error && (
            <p className="text-sm text-destructive">{error}</p>
          )}
        </div>

        <div className="mt-6 flex justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
            disabled={creating}
            className="rounded-md border border-border px-4 py-2 text-sm font-medium text-foreground hover:bg-muted disabled:opacity-50"
          >
            {t("send.cancel")}
          </button>
          <button
            type="submit"
            disabled={creating}
            className="flex items-center gap-1.5 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            {creating && <Spinner size="sm" />}
            {t("send.createButton")}
          </button>
        </div>
      </form>
    </div>
  );
}
