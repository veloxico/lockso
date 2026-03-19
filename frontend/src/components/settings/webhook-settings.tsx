import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Bell, Plus, Trash2, TestTube, Check, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { webhookApi, type WebhookView } from "@/api/admin";

const PROVIDERS = [
  { value: "telegram", label: "Telegram", placeholder: "bot_token|chat_id" },
  { value: "slack", label: "Slack", placeholder: "https://hooks.slack.com/services/..." },
  { value: "discord", label: "Discord", placeholder: "https://discord.com/api/webhooks/..." },
  { value: "custom", label: "Custom URL", placeholder: "https://your-server.com/webhook" },
];

const ALL_EVENTS = [
  "user.login", "user.login_failed", "user.register", "user.blocked",
  "item.created", "item.updated", "item.trashed", "item.restored",
  "vault.created", "vault.deleted",
  "send.created", "send.accessed",
  "trash.emptied", "settings.updated",
];

export function WebhookSettings() {
  const { t } = useTranslation();
  const [webhooks, setWebhooks] = useState<WebhookView[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [testing, setTesting] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<{ id: string; ok: boolean } | null>(null);

  const load = useCallback(async () => {
    try {
      const data = await webhookApi.list();
      setWebhooks(data);
    } catch {
      /* ignore */
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleDelete = async (id: string) => {
    if (!confirm(t("webhooks.deleteConfirm"))) return;
    try {
      await webhookApi.delete(id);
      setWebhooks((prev) => prev.filter((w) => w.id !== id));
    } catch { /* ignore */ }
  };

  const handleTest = async (id: string) => {
    setTesting(id);
    setTestResult(null);
    try {
      await webhookApi.test(id);
      setTestResult({ id, ok: true });
    } catch {
      setTestResult({ id, ok: false });
    } finally {
      setTesting(null);
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      const updated = await webhookApi.update(id, { isEnabled: enabled });
      setWebhooks((prev) => prev.map((w) => (w.id === id ? updated : w)));
    } catch { /* ignore */ }
  };

  if (loading) return <div className="flex justify-center py-8"><Spinner size="lg" /></div>;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">{t("webhooks.description")}</p>
        <Button size="sm" onClick={() => setShowCreate(true)}>
          <Plus className="h-4 w-4 mr-1" />
          {t("webhooks.add")}
        </Button>
      </div>

      {showCreate && (
        <CreateWebhookForm
          onCreated={(wh) => { setWebhooks((p) => [wh, ...p]); setShowCreate(false); }}
          onCancel={() => setShowCreate(false)}
        />
      )}

      {webhooks.length === 0 && !showCreate ? (
        <div className="flex flex-col items-center py-12 text-center">
          <Bell className="h-10 w-10 text-muted-foreground" />
          <p className="mt-3 text-sm text-muted-foreground">{t("webhooks.empty")}</p>
        </div>
      ) : (
        <div className="space-y-2">
          {webhooks.map((wh) => (
            <div key={wh.id} className="flex items-center gap-3 rounded-md border border-border p-3">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium">{wh.name}</span>
                  <span className="text-[10px] rounded-full bg-muted px-1.5 py-0.5 text-muted-foreground uppercase">
                    {wh.provider}
                  </span>
                  {!wh.isEnabled && (
                    <span className="text-[10px] rounded-full bg-destructive/10 px-1.5 py-0.5 text-destructive">
                      {t("webhooks.disabled")}
                    </span>
                  )}
                </div>
                <p className="text-xs text-muted-foreground mt-0.5 truncate">{wh.urlMasked}</p>
                <p className="text-[10px] text-muted-foreground/60 mt-0.5">
                  {wh.events.length} {t("webhooks.events")}
                </p>
              </div>

              <div className="flex items-center gap-1 shrink-0">
                {testResult?.id === wh.id && (
                  testResult.ok
                    ? <Check className="h-4 w-4 text-green-500" />
                    : <X className="h-4 w-4 text-destructive" />
                )}
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => handleTest(wh.id)}
                  disabled={testing === wh.id}
                  title={t("webhooks.test")}
                >
                  {testing === wh.id ? <Spinner size="sm" /> : <TestTube className="h-4 w-4" />}
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => handleToggle(wh.id, !wh.isEnabled)}
                  title={wh.isEnabled ? t("webhooks.disable") : t("webhooks.enable")}
                >
                  <Bell className={`h-4 w-4 ${wh.isEnabled ? "text-green-500" : "text-muted-foreground"}`} />
                </Button>
                <Button variant="ghost" size="icon" onClick={() => handleDelete(wh.id)}>
                  <Trash2 className="h-4 w-4 text-destructive" />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function CreateWebhookForm({
  onCreated,
  onCancel,
}: {
  onCreated: (wh: WebhookView) => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [provider, setProvider] = useState("telegram");
  const [url, setUrl] = useState("");
  const [events, setEvents] = useState<string[]>(["user.login", "user.login_failed", "item.created", "item.trashed"]);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const providerInfo = PROVIDERS.find((p) => p.value === provider);

  const toggleEvent = (e: string) => {
    setEvents((prev) => prev.includes(e) ? prev.filter((x) => x !== e) : [...prev, e]);
  };

  const handleSubmit = async () => {
    if (!name.trim() || !url.trim()) return;
    setSaving(true);
    setError("");
    try {
      const wh = await webhookApi.create({ name: name.trim(), provider, url: url.trim(), events });
      onCreated(wh);
    } catch {
      setError(t("webhooks.createError"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="rounded-md border border-border p-4 space-y-3">
      <div className="grid grid-cols-2 gap-3">
        <div>
          <Label>{t("webhooks.name")}</Label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="mt-1 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
            placeholder={t("webhooks.namePlaceholder")}
          />
        </div>
        <div>
          <Label>{t("webhooks.provider")}</Label>
          <select
            value={provider}
            onChange={(e) => setProvider(e.target.value)}
            className="mt-1 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
          >
            {PROVIDERS.map((p) => (
              <option key={p.value} value={p.value}>{p.label}</option>
            ))}
          </select>
        </div>
      </div>

      <div>
        <Label>{t("webhooks.url")}</Label>
        <input
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          className="mt-1 w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono"
          placeholder={providerInfo?.placeholder}
        />
      </div>

      <div>
        <Label>{t("webhooks.selectEvents")}</Label>
        <div className="mt-1 flex flex-wrap gap-1.5">
          {ALL_EVENTS.map((e) => (
            <button
              key={e}
              onClick={() => toggleEvent(e)}
              className={`rounded-full px-2 py-0.5 text-[11px] border transition-colors ${
                events.includes(e)
                  ? "border-primary bg-primary/10 text-primary"
                  : "border-border text-muted-foreground hover:border-muted-foreground/50"
              }`}
            >
              {e}
            </button>
          ))}
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <div className="flex justify-end gap-2">
        <Button variant="outline" size="sm" onClick={onCancel}>{t("vault.cancel")}</Button>
        <Button size="sm" onClick={handleSubmit} disabled={saving || !name.trim() || !url.trim()}>
          {saving ? <Spinner size="sm" /> : t("webhooks.create")}
        </Button>
      </div>
    </div>
  );
}
