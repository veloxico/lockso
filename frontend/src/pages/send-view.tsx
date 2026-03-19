import { useState, useEffect, useCallback } from "react";
import { useParams } from "react-router";
import { useTranslation } from "react-i18next";
import { Shield, Lock, Eye, Copy, Check, AlertTriangle } from "lucide-react";
import { Spinner } from "@/components/ui/spinner";
import { LocksoLogo } from "@/components/lockso-logo";
import { ThemeToggle } from "@/components/theme-toggle";
import { publicSendApi } from "@/api/vaults";
import { decryptSendPayload } from "@/lib/send-crypto";

type ViewState =
  | "loading"
  | "passphrase"
  | "ready"
  | "revealed"
  | "error"
  | "not_found";

export function SendViewPage() {
  const { t } = useTranslation();
  const { accessId } = useParams<{ accessId: string }>();
  const [state, setState] = useState<ViewState>("loading");
  const [needsPassphrase, setNeedsPassphrase] = useState(false);
  const [passphrase, setPassphrase] = useState("");
  const [plaintext, setPlaintext] = useState("");
  const [error, setError] = useState("");
  const [copied, setCopied] = useState(false);
  const [revealing, setRevealing] = useState(false);

  // Get encryption key from URL fragment
  const key = typeof window !== "undefined" ? window.location.hash.slice(1) : "";

  const loadMeta = useCallback(async () => {
    if (!accessId) {
      setState("not_found");
      return;
    }
    try {
      const meta = await publicSendApi.meta(accessId);
      if (meta.hasPassphrase) {
        setNeedsPassphrase(true);
        setState("passphrase");
      } else {
        setState("ready");
      }
    } catch {
      setState("not_found");
    }
  }, [accessId]);

  useEffect(() => {
    loadMeta();
  }, [loadMeta]);

  const handleReveal = async (e?: React.FormEvent) => {
    e?.preventDefault();
    if (!accessId || !key) {
      setError(t("send.errorNoKey"));
      setState("error");
      return;
    }

    setRevealing(true);
    setError("");

    try {
      const data = await publicSendApi.access(
        accessId,
        needsPassphrase ? passphrase : undefined,
      );

      // Decrypt client-side
      const decrypted = await decryptSendPayload(data.ciphertextB64, key);
      setPlaintext(decrypted);
      setState("revealed");
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "error";
      if (msg === "passphrase_required" || msg === "invalid_passphrase") {
        setError(t("send.errorWrongPassphrase"));
        setState("passphrase");
      } else if (msg === "not_found" || msg === "expired" || msg === "consumed") {
        setState("not_found");
      } else {
        setError(t("send.errorDecryptFailed"));
        setState("error");
      }
    } finally {
      setRevealing(false);
    }
  };

  const handleCopy = async () => {
    await navigator.clipboard.writeText(plaintext);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex min-h-screen flex-col bg-background">
      {/* Minimal header */}
      <header className="flex items-center justify-between border-b border-border px-4 py-3">
        <LocksoLogo size="sm" />
        <ThemeToggle />
      </header>

      <main className="flex flex-1 items-center justify-center p-4">
        <div className="w-full max-w-md">
          {state === "loading" && (
            <div className="flex flex-col items-center gap-3">
              <Spinner size="lg" />
            </div>
          )}

          {state === "not_found" && (
            <div className="flex flex-col items-center text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-muted">
                <AlertTriangle className="h-8 w-8 text-muted-foreground" />
              </div>
              <h2 className="mt-4 text-lg font-semibold">{t("send.notFound")}</h2>
              <p className="mt-2 text-sm text-muted-foreground">
                {t("send.notFoundDescription")}
              </p>
            </div>
          )}

          {state === "error" && (
            <div className="flex flex-col items-center text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-destructive/10">
                <AlertTriangle className="h-8 w-8 text-destructive" />
              </div>
              <h2 className="mt-4 text-lg font-semibold text-destructive">
                {t("send.errorTitle")}
              </h2>
              <p className="mt-2 text-sm text-muted-foreground">{error}</p>
            </div>
          )}

          {state === "passphrase" && (
            <div className="rounded-lg border border-border bg-card p-6 shadow-sm">
              <div className="flex flex-col items-center text-center">
                <div className="flex h-14 w-14 items-center justify-center rounded-full bg-primary/10">
                  <Lock className="h-7 w-7 text-primary" />
                </div>
                <h2 className="mt-3 text-lg font-semibold">
                  {t("send.passphraseRequired")}
                </h2>
                <p className="mt-1 text-sm text-muted-foreground">
                  {t("send.passphraseRequiredDescription")}
                </p>
              </div>

              <form onSubmit={handleReveal} className="mt-4">
                <input
                  type="password"
                  value={passphrase}
                  onChange={(e) => setPassphrase(e.target.value)}
                  className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                  placeholder={t("send.enterPassphrase")}
                  autoFocus
                />
                {error && (
                  <p className="mt-1 text-sm text-destructive">{error}</p>
                )}
                <button
                  type="submit"
                  disabled={revealing}
                  className="mt-3 flex w-full items-center justify-center gap-1.5 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                  {revealing ? <Spinner size="sm" /> : <Eye className="h-4 w-4" />}
                  {t("send.reveal")}
                </button>
              </form>
            </div>
          )}

          {state === "ready" && (
            <div className="rounded-lg border border-border bg-card p-6 shadow-sm">
              <div className="flex flex-col items-center text-center">
                <div className="flex h-14 w-14 items-center justify-center rounded-full bg-primary/10">
                  <Shield className="h-7 w-7 text-primary" />
                </div>
                <h2 className="mt-3 text-lg font-semibold">
                  {t("send.readyTitle")}
                </h2>
                <p className="mt-1 text-sm text-muted-foreground">
                  {t("send.readyDescription")}
                </p>
              </div>

              <button
                onClick={() => handleReveal()}
                disabled={revealing}
                className="mt-4 flex w-full items-center justify-center gap-1.5 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              >
                {revealing ? <Spinner size="sm" /> : <Eye className="h-4 w-4" />}
                {t("send.reveal")}
              </button>
            </div>
          )}

          {state === "revealed" && (
            <div className="rounded-lg border border-border bg-card p-6 shadow-sm">
              <div className="flex items-center justify-between">
                <h2 className="text-lg font-semibold">{t("send.revealedTitle")}</h2>
                <button
                  onClick={handleCopy}
                  className="flex items-center gap-1 rounded-md border border-border px-2.5 py-1 text-xs font-medium hover:bg-muted"
                >
                  {copied ? (
                    <>
                      <Check className="h-3 w-3 text-emerald-500" />
                      {t("send.copied")}
                    </>
                  ) : (
                    <>
                      <Copy className="h-3 w-3" />
                      {t("send.copy")}
                    </>
                  )}
                </button>
              </div>
              <div className="mt-3 rounded-md bg-muted p-4 text-sm font-mono text-foreground whitespace-pre-wrap break-all">
                {plaintext}
              </div>
              <p className="mt-3 text-center text-xs text-muted-foreground">
                {t("send.revealedWarning")}
              </p>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
