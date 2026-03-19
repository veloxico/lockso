import { useState, useRef, useEffect, type FormEvent } from "react";
import { useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import { Shield } from "lucide-react";
import { AuthLayout } from "@/components/layout/auth-layout";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { LocksoLogo } from "@/components/lockso-logo";
import { totpApi } from "@/api/totp";
import { useAuthStore } from "@/stores/auth";

export function TwoFactorPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const confirmTwoFactor = useAuthStore((s) => s.confirmTwoFactor);
  const inputRef = useRef<HTMLInputElement>(null);

  const [code, setCode] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (code.length < 6) return;

    setLoading(true);
    setError("");

    try {
      const result = await totpApi.verify(code.replace(/[^0-9A-Za-z-]/g, ""));
      if (result.verified) {
        confirmTwoFactor();
        navigate("/");
      } else {
        setError(t("twoFactor.invalidCode"));
        setCode("");
        inputRef.current?.focus();
      }
    } catch {
      setError(t("twoFactor.invalidCode"));
      setCode("");
      inputRef.current?.focus();
    } finally {
      setLoading(false);
    }
  };

  return (
    <AuthLayout>
      <div className="flex flex-col items-center gap-4">
        <LocksoLogo size="md" />

        <div className="flex items-center gap-2">
          <Shield className="h-5 w-5 text-primary" />
          <h1 className="text-xl font-bold">{t("twoFactor.title")}</h1>
        </div>

        <p className="text-sm text-muted-foreground text-center">
          {t("twoFactor.description")}
        </p>

        <form onSubmit={handleSubmit} className="w-full space-y-4 mt-2">
          <div className="space-y-2">
            <Label htmlFor="code">{t("twoFactor.codeLabel")}</Label>
            <Input
              ref={inputRef}
              id="code"
              value={code}
              onChange={(e) => setCode(e.target.value)}
              placeholder="000000"
              maxLength={9}
              autoComplete="one-time-code"
              inputMode="numeric"
              className="text-center text-lg tracking-[0.3em] font-mono"
            />
            <p className="text-xs text-muted-foreground text-center">
              {t("twoFactor.recoveryHint")}
            </p>
          </div>

          {error && <p className="text-sm text-destructive text-center">{error}</p>}

          <Button type="submit" className="w-full" disabled={loading || code.length < 6}>
            {loading && <Spinner size="sm" />}
            {t("twoFactor.verify")}
          </Button>
        </form>
      </div>
    </AuthLayout>
  );
}
