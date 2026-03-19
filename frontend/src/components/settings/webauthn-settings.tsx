import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { KeyRound, Plus, Trash2, Pencil, Fingerprint, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Spinner } from "@/components/ui/spinner";
import {
  webauthnApi,
  registerCredential,
  type WebAuthnCredentialView,
} from "@/api/webauthn";

export function WebAuthnSettings() {
  const { t } = useTranslation();
  const [credentials, setCredentials] = useState<WebAuthnCredentialView[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [registering, setRegistering] = useState(false);
  const [newDeviceName, setNewDeviceName] = useState("");
  const [showAddForm, setShowAddForm] = useState(false);
  const [success, setSuccess] = useState("");

  const load = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const creds = await webauthnApi.listCredentials();
      setCredentials(creds);
    } catch {
      setError(t("settings.errorLoadFailed"));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    load();
  }, [load]);

  const handleRegister = async () => {
    if (!newDeviceName.trim()) return;
    setRegistering(true);
    setError("");
    try {
      const cred = await registerCredential(newDeviceName.trim());
      setCredentials((prev) => [cred, ...prev]);
      setNewDeviceName("");
      setShowAddForm(false);
      setSuccess(t("webauthn.registered"));
      setTimeout(() => setSuccess(""), 3000);
    } catch (err) {
      const msg = err instanceof Error ? err.message : t("settings.errorActionFailed");
      setError(msg);
    } finally {
      setRegistering(false);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await webauthnApi.deleteCredential(id);
      setCredentials((prev) => prev.filter((c) => c.id !== id));
    } catch {
      setError(t("settings.errorActionFailed"));
    }
  };

  const isWebAuthnSupported =
    typeof window !== "undefined" &&
    window.PublicKeyCredential !== undefined;

  return (
    <div className="max-w-2xl space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Fingerprint className="h-5 w-5 text-muted-foreground" />
          <div>
            <h2 className="text-lg font-semibold">{t("webauthn.title")}</h2>
            <p className="text-sm text-muted-foreground">
              {t("webauthn.description")}
            </p>
          </div>
        </div>

        {!showAddForm && isWebAuthnSupported && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowAddForm(true)}
          >
            <Plus className="mr-1.5 h-3.5 w-3.5" />
            {t("webauthn.addKey")}
          </Button>
        )}
      </div>

      {!isWebAuthnSupported && (
        <p className="text-sm text-destructive">{t("webauthn.notSupported")}</p>
      )}

      {error && <p className="text-sm text-destructive">{error}</p>}
      {success && (
        <p className="flex items-center gap-1 text-sm text-success">
          <Check className="h-3.5 w-3.5" /> {success}
        </p>
      )}

      {/* Add key form */}
      {showAddForm && (
        <div className="rounded-lg border border-border p-4 space-y-3">
          <p className="text-sm font-medium">{t("webauthn.addTitle")}</p>
          <div className="flex gap-2">
            <Input
              value={newDeviceName}
              onChange={(e) => setNewDeviceName(e.target.value)}
              placeholder={t("webauthn.deviceNamePlaceholder")}
              maxLength={255}
              autoFocus
            />
            <Button
              onClick={handleRegister}
              disabled={registering || !newDeviceName.trim()}
            >
              {registering ? <Spinner size="sm" /> : t("webauthn.register")}
            </Button>
            <Button
              variant="ghost"
              onClick={() => {
                setShowAddForm(false);
                setNewDeviceName("");
              }}
            >
              {t("vault.cancel")}
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">
            {t("webauthn.addHint")}
          </p>
        </div>
      )}

      {/* Credentials list */}
      {loading ? (
        <div className="flex justify-center py-12">
          <Spinner size="md" />
        </div>
      ) : credentials.length === 0 ? (
        <p className="text-sm text-muted-foreground py-8 text-center">
          {t("webauthn.noKeys")}
        </p>
      ) : (
        <div className="space-y-3">
          {credentials.map((cred) => (
            <CredentialCard
              key={cred.id}
              credential={cred}
              onDelete={handleDelete}
              onRename={async (id, name) => {
                await webauthnApi.renameCredential(id, name);
                setCredentials((prev) =>
                  prev.map((c) =>
                    c.id === id ? { ...c, deviceName: name } : c,
                  ),
                );
              }}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function CredentialCard({
  credential,
  onDelete,
  onRename,
}: {
  credential: WebAuthnCredentialView;
  onDelete: (id: string) => void;
  onRename: (id: string, name: string) => void;
}) {
  const { t } = useTranslation();
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(credential.deviceName);

  const handleRename = () => {
    if (name.trim() && name !== credential.deviceName) {
      onRename(credential.id, name.trim());
    }
    setEditing(false);
  };

  return (
    <div className="flex items-center justify-between rounded-lg border border-border p-4">
      <div className="flex items-center gap-3">
        <KeyRound className="h-5 w-5 text-muted-foreground" />
        <div>
          {editing ? (
            <div className="flex gap-2">
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleRename()}
                className="h-7 w-48 text-sm"
                autoFocus
              />
              <Button variant="ghost" size="sm" onClick={handleRename}>
                <Check className="h-3.5 w-3.5" />
              </Button>
            </div>
          ) : (
            <p className="text-sm font-medium">{credential.deviceName}</p>
          )}
          <div className="flex items-center gap-3 text-xs text-muted-foreground">
            <span>
              {t("webauthn.added")}{" "}
              {new Date(credential.createdAt).toLocaleDateString()}
            </span>
            {credential.lastUsedAt && (
              <span>
                {t("webauthn.lastUsed")}{" "}
                {new Date(credential.lastUsedAt).toLocaleDateString()}
              </span>
            )}
          </div>
        </div>
      </div>

      <div className="flex gap-1">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setEditing(true)}
          className="h-8 w-8"
        >
          <Pencil className="h-3.5 w-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          onClick={() => onDelete(credential.id)}
          className="h-8 w-8 text-destructive hover:text-destructive"
        >
          <Trash2 className="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  );
}
