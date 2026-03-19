import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Shield, ShieldCheck, ShieldOff, Copy, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { Alert } from "@/components/ui/alert";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogContent,
  DialogFooter,
} from "@/components/ui/dialog";
import { totpApi, type TotpSetupResponse, type TotpStatus } from "@/api/totp";

export function TwoFactorSettings() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<TotpStatus | null>(null);
  const [loading, setLoading] = useState(true);

  // Setup state
  const [setupData, setSetupData] = useState<TotpSetupResponse | null>(null);
  const [setupDialogOpen, setSetupDialogOpen] = useState(false);
  const [verifyCode, setVerifyCode] = useState("");
  const [setupStep, setSetupStep] = useState<"qr" | "codes">("qr");
  const [enabling, setEnabling] = useState(false);

  // Disable state
  const [disableDialogOpen, setDisableDialogOpen] = useState(false);
  const [disableCode, setDisableCode] = useState("");
  const [disabling, setDisabling] = useState(false);

  const [error, setError] = useState("");
  const [copiedCodes, setCopiedCodes] = useState(false);
  const copyTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(copyTimerRef.current), []);

  const loadStatus = useCallback(async () => {
    try {
      const s = await totpApi.getStatus();
      setStatus(s);
    } catch {
      // Ignore
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  const handleStartSetup = async () => {
    setError("");
    try {
      const data = await totpApi.setup();
      setSetupData(data);
      setSetupStep("qr");
      setVerifyCode("");
      setSetupDialogOpen(true);
    } catch {
      setError(t("twoFactor.errorSetupFailed"));
    }
  };

  const handleEnable = async () => {
    if (!setupData || verifyCode.length < 6) return;
    setEnabling(true);
    setError("");

    try {
      const result = await totpApi.enable(setupData.secret, verifyCode, setupData.recoveryCodes);
      setStatus(result);
      setSetupStep("codes");
    } catch {
      setError(t("twoFactor.invalidCode"));
    } finally {
      setEnabling(false);
    }
  };

  const handleDisable = async () => {
    if (disableCode.length < 6) return;
    setDisabling(true);
    setError("");

    try {
      const result = await totpApi.disable(disableCode);
      setStatus(result);
      setDisableDialogOpen(false);
      setDisableCode("");
    } catch {
      setError(t("twoFactor.invalidCode"));
    } finally {
      setDisabling(false);
    }
  };

  const handleCopyCodes = () => {
    if (!setupData) return;
    navigator.clipboard.writeText(setupData.recoveryCodes.join("\n"));
    setCopiedCodes(true);
    clearTimeout(copyTimerRef.current);
    copyTimerRef.current = setTimeout(() => setCopiedCodes(false), 3000);
  };

  if (loading) {
    return (
      <div className="flex justify-center py-12">
        <Spinner size="md" />
      </div>
    );
  }

  return (
    <div className="max-w-lg space-y-6">
      <div className="flex items-center gap-2">
        <Shield className="h-5 w-5 text-muted-foreground" />
        <div>
          <h2 className="text-lg font-semibold">{t("twoFactor.settingsTitle")}</h2>
          <p className="text-sm text-muted-foreground">
            {t("twoFactor.settingsDescription")}
          </p>
        </div>
      </div>

      {/* Status */}
      <div className="flex items-center justify-between rounded-lg border border-border p-4">
        <div className="flex items-center gap-3">
          {status?.isEnabled ? (
            <ShieldCheck className="h-6 w-6 text-green-500" />
          ) : (
            <ShieldOff className="h-6 w-6 text-muted-foreground" />
          )}
          <div>
            <p className="font-medium">
              {status?.isEnabled
                ? t("twoFactor.enabled")
                : t("twoFactor.disabled")}
            </p>
            {status?.isEnabled && (
              <p className="text-xs text-muted-foreground">
                {t("twoFactor.recoveryRemaining", {
                  count: status.recoveryCodesRemaining,
                })}
              </p>
            )}
          </div>
        </div>

        {status?.isEnabled ? (
          <Button
            variant="destructive"
            size="sm"
            onClick={() => {
              setDisableCode("");
              setError("");
              setDisableDialogOpen(true);
            }}
          >
            {t("twoFactor.disable")}
          </Button>
        ) : (
          <Button size="sm" onClick={handleStartSetup}>
            {t("twoFactor.enable")}
          </Button>
        )}
      </div>

      {error && !setupDialogOpen && !disableDialogOpen && (
        <p className="text-sm text-destructive">{error}</p>
      )}

      {/* Setup dialog */}
      <Dialog open={setupDialogOpen} onClose={() => setSetupDialogOpen(false)}>
        <DialogHeader>
          <DialogTitle>
            {setupStep === "qr"
              ? t("twoFactor.setupTitle")
              : t("twoFactor.recoveryTitle")}
          </DialogTitle>
          <DialogDescription>
            {setupStep === "qr"
              ? t("twoFactor.setupDescription")
              : t("twoFactor.recoveryDescription")}
          </DialogDescription>
        </DialogHeader>

        <DialogContent>
          {setupStep === "qr" ? (
            <div className="space-y-4">
              {/* Secret for manual entry */}
              <div className="space-y-2">
                <Label>{t("twoFactor.secretLabel")}</Label>
                <div className="rounded-md bg-muted p-3 font-mono text-sm tracking-wider break-all select-all">
                  {setupData?.secret}
                </div>
                <p className="text-xs text-muted-foreground">
                  {t("twoFactor.secretHint")}
                </p>
              </div>

              {/* QR URI for debug / manual QR generation */}
              <details className="text-xs text-muted-foreground">
                <summary className="cursor-pointer">
                  {t("twoFactor.showUri")}
                </summary>
                <code className="mt-1 block break-all text-[10px]">
                  {setupData?.otpauthUri}
                </code>
              </details>

              {/* Verification code */}
              <div className="space-y-2">
                <Label>{t("twoFactor.enterCode")}</Label>
                <Input
                  value={verifyCode}
                  onChange={(e) => setVerifyCode(e.target.value)}
                  placeholder="000000"
                  maxLength={6}
                  inputMode="numeric"
                  className="text-center text-lg tracking-[0.3em] font-mono"
                />
              </div>

              {error && <p className="text-sm text-destructive">{error}</p>}
            </div>
          ) : (
            <div className="space-y-4">
              <Alert variant="destructive">
                <p className="text-sm font-medium">
                  {t("twoFactor.recoveryWarning")}
                </p>
              </Alert>

              <div className="rounded-md border border-border p-4 bg-muted/30">
                <div className="grid grid-cols-2 gap-2 font-mono text-sm">
                  {setupData?.recoveryCodes.map((code, i) => (
                    <div key={i} className="text-center py-1">
                      {code}
                    </div>
                  ))}
                </div>
              </div>

              <Button
                variant="outline"
                size="sm"
                onClick={handleCopyCodes}
                className="w-full"
              >
                {copiedCodes ? (
                  <Check className="h-4 w-4 text-green-500" />
                ) : (
                  <Copy className="h-4 w-4" />
                )}
                {copiedCodes
                  ? t("twoFactor.copied")
                  : t("twoFactor.copyCodes")}
              </Button>
            </div>
          )}
        </DialogContent>

        <DialogFooter>
          {setupStep === "qr" ? (
            <>
              <Button
                variant="outline"
                onClick={() => setSetupDialogOpen(false)}
              >
                {t("vault.cancel")}
              </Button>
              <Button
                onClick={handleEnable}
                disabled={enabling || verifyCode.length < 6}
              >
                {enabling && <Spinner size="sm" />}
                {t("twoFactor.verify")}
              </Button>
            </>
          ) : (
            <Button onClick={() => setSetupDialogOpen(false)}>
              {t("twoFactor.done")}
            </Button>
          )}
        </DialogFooter>
      </Dialog>

      {/* Disable dialog */}
      <Dialog
        open={disableDialogOpen}
        onClose={() => setDisableDialogOpen(false)}
      >
        <DialogHeader>
          <DialogTitle>{t("twoFactor.disableTitle")}</DialogTitle>
          <DialogDescription>
            {t("twoFactor.disableDescription")}
          </DialogDescription>
        </DialogHeader>
        <DialogContent>
          <div className="space-y-3">
            <Label>{t("twoFactor.enterCode")}</Label>
            <Input
              value={disableCode}
              onChange={(e) => setDisableCode(e.target.value)}
              placeholder="000000"
              maxLength={9}
              inputMode="numeric"
              className="text-center text-lg tracking-[0.3em] font-mono"
            />
            {error && <p className="text-sm text-destructive">{error}</p>}
          </div>
        </DialogContent>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => setDisableDialogOpen(false)}
          >
            {t("vault.cancel")}
          </Button>
          <Button
            variant="destructive"
            onClick={handleDisable}
            disabled={disabling || disableCode.length < 6}
          >
            {disabling && <Spinner size="sm" />}
            {t("twoFactor.disable")}
          </Button>
        </DialogFooter>
      </Dialog>
    </div>
  );
}
