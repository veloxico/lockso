import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Key, Plus, Trash2, Copy, Check, Clock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { apiKeyApi, type ApiKeyView, type ApiKeyCreated } from "@/api/admin";

export function ApiKeySettings() {
  const { t } = useTranslation();
  const [keys, setKeys] = useState<ApiKeyView[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [newKey, setNewKey] = useState<ApiKeyCreated | null>(null);
  const [copied, setCopied] = useState(false);

  const load = useCallback(async () => {
    try {
      const data = await apiKeyApi.list();
      setKeys(data);
    } catch {
      /* ignore */
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleDelete = async (id: string) => {
    if (!confirm(t("apiKeys.deleteConfirm"))) return;
    try {
      await apiKeyApi.delete(id);
      setKeys((prev) => prev.filter((k) => k.id !== id));
    } catch { /* ignore */ }
  };

  const handleCopy = async (key: string) => {
    await navigator.clipboard.writeText(key);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (loading) return <div className="flex justify-center py-8"><Spinner size="lg" /></div>;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">{t("apiKeys.description")}</p>
        <Button size="sm" onClick={() => { setShowCreate(true); setNewKey(null); }}>
          <Plus className="h-4 w-4 mr-1" />
          {t("apiKeys.add")}
        </Button>
      </div>

      {/* New key reveal */}
      {newKey && (
        <div className="rounded-md border border-green-500/30 bg-green-500/5 p-4 space-y-2">
          <p className="text-sm font-medium text-green-600 dark:text-green-400">
            {t("apiKeys.createdSuccess")}
          </p>
          <p className="text-xs text-muted-foreground">{t("apiKeys.createdWarning")}</p>
          <div className="flex items-center gap-2">
            <code className="flex-1 rounded bg-muted px-3 py-2 text-xs font-mono break-all select-all">
              {newKey.key}
            </code>
            <Button variant="outline" size="icon" onClick={() => handleCopy(newKey.key)}>
              {copied ? <Check className="h-4 w-4 text-green-500" /> : <Copy className="h-4 w-4" />}
            </Button>
          </div>
        </div>
      )}

      {showCreate && !newKey && (
        <CreateKeyForm
          onCreated={(k) => { setNewKey(k); load(); }}
          onCancel={() => setShowCreate(false)}
        />
      )}

      {keys.length === 0 && !showCreate ? (
        <div className="flex flex-col items-center py-12 text-center">
          <Key className="h-10 w-10 text-muted-foreground" />
          <p className="mt-3 text-sm text-muted-foreground">{t("apiKeys.empty")}</p>
        </div>
      ) : (
        <div className="space-y-2">
          {keys.map((key) => (
            <div key={key.id} className="flex items-center gap-3 rounded-md border border-border p-3">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium">{key.name}</span>
                  <code className="text-[10px] rounded bg-muted px-1.5 py-0.5 text-muted-foreground font-mono">
                    {key.keyPrefix}••••
                  </code>
                  <span className={`text-[10px] rounded-full px-1.5 py-0.5 ${
                    key.permission === "read_write"
                      ? "bg-amber-500/10 text-amber-600"
                      : "bg-blue-500/10 text-blue-600"
                  }`}>
                    {key.permission}
                  </span>
                </div>
                <div className="flex items-center gap-3 mt-0.5 text-[10px] text-muted-foreground/60">
                  {key.lastUsedAt && (
                    <span className="flex items-center gap-0.5">
                      <Clock className="h-2.5 w-2.5" />
                      {t("apiKeys.lastUsed")}: {new Date(key.lastUsedAt).toLocaleDateString()}
                    </span>
                  )}
                  {key.expiresAt && (
                    <span>
                      {t("apiKeys.expires")}: {new Date(key.expiresAt).toLocaleDateString()}
                    </span>
                  )}
                </div>
              </div>
              <Button variant="ghost" size="icon" onClick={() => handleDelete(key.id)}>
                <Trash2 className="h-4 w-4 text-destructive" />
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function CreateKeyForm({
  onCreated,
  onCancel,
}: {
  onCreated: (k: ApiKeyCreated) => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [permission, setPermission] = useState("read");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const handleSubmit = async () => {
    if (!name.trim()) return;
    setSaving(true);
    setError("");
    try {
      const key = await apiKeyApi.create({ name: name.trim(), permission });
      onCreated(key);
    } catch {
      setError(t("apiKeys.createError"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="rounded-md border border-border p-4 space-y-3">
      <div className="grid grid-cols-2 gap-3">
        <div>
          <Label>{t("apiKeys.name")}</Label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="mt-1 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
            placeholder={t("apiKeys.namePlaceholder")}
          />
        </div>
        <div>
          <Label>{t("apiKeys.permission")}</Label>
          <select
            value={permission}
            onChange={(e) => setPermission(e.target.value)}
            className="mt-1 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
          >
            <option value="read">{t("apiKeys.permRead")}</option>
            <option value="read_write">{t("apiKeys.permReadWrite")}</option>
          </select>
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <div className="flex justify-end gap-2">
        <Button variant="outline" size="sm" onClick={onCancel}>{t("vault.cancel")}</Button>
        <Button size="sm" onClick={handleSubmit} disabled={saving || !name.trim()}>
          {saving ? <Spinner size="sm" /> : t("apiKeys.create")}
        </Button>
      </div>
    </div>
  );
}
