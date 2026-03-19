import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Save, Shield, Clock, Lock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { settingsApi } from "@/api/admin";
import type {
  SessionSettings,
  PasswordComplexity,
  UserLockoutSettings,
} from "@/types/admin";

export function SecuritySettings() {
  const { t } = useTranslation();
  const [session, setSession] = useState<SessionSettings>({
    accessTokenTtl: 3600,
    refreshTokenTtl: 2592000,
    inactivityTtl: 1800,
    csrfTokenTtl: 3600,
  });
  const [passwordRules, setPasswordRules] = useState<PasswordComplexity>({
    minLength: 8,
    requireUppercase: true,
    requireLowercase: true,
    requireDigits: true,
    requireSpecial: false,
  });
  const [lockout, setLockout] = useState<UserLockoutSettings>({
    enabled: true,
    maxAttempts: 7,
    windowSeconds: 180,
    lockoutSeconds: 60,
  });

  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const data = await settingsApi.get();
      if (data.session) setSession(data.session);
      if (data.authPasswordComplexity) setPasswordRules(data.authPasswordComplexity);
      if (data.userLockout) setLockout(data.userLockout);
    } catch {
      // Use defaults
    } finally {
      setLoading(false);
    }
  };

  const saveCategory = async (category: string, value: Record<string, unknown>) => {
    setSaving(category);
    setError("");
    setSaved(null);

    try {
      await settingsApi.updateCategory(category, value);
      setSaved(category);
      setTimeout(() => setSaved(null), 3000);
    } catch {
      setError(t("settings.errorSaveFailed"));
    } finally {
      setSaving(null);
    }
  };

  if (loading) {
    return (
      <div className="flex justify-center py-12">
        <Spinner size="md" />
      </div>
    );
  }

  return (
    <div className="max-w-lg space-y-10">
      {/* Session settings */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <Clock className="h-5 w-5 text-muted-foreground" />
          <div>
            <h2 className="text-lg font-semibold">{t("settings.sessionTitle")}</h2>
            <p className="text-sm text-muted-foreground">{t("settings.sessionDescription")}</p>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <Label>{t("settings.accessTokenTtl")}</Label>
            <Input
              type="number"
              value={session.accessTokenTtl}
              onChange={(e) =>
                setSession({ ...session, accessTokenTtl: Number(e.target.value) })
              }
              min={300}
              max={86400}
            />
            <p className="text-xs text-muted-foreground">{t("settings.seconds")}</p>
          </div>
          <div className="space-y-2">
            <Label>{t("settings.refreshTokenTtl")}</Label>
            <Input
              type="number"
              value={session.refreshTokenTtl}
              onChange={(e) =>
                setSession({ ...session, refreshTokenTtl: Number(e.target.value) })
              }
              min={3600}
              max={7776000}
            />
            <p className="text-xs text-muted-foreground">{t("settings.seconds")}</p>
          </div>
          <div className="space-y-2">
            <Label>{t("settings.inactivityTtl")}</Label>
            <Input
              type="number"
              value={session.inactivityTtl}
              onChange={(e) =>
                setSession({ ...session, inactivityTtl: Number(e.target.value) })
              }
              min={0}
              max={86400}
            />
            <p className="text-xs text-muted-foreground">{t("settings.zeroToDisable")}</p>
          </div>
          <div className="space-y-2">
            <Label>{t("settings.csrfTokenTtl")}</Label>
            <Input
              type="number"
              value={session.csrfTokenTtl}
              onChange={(e) =>
                setSession({ ...session, csrfTokenTtl: Number(e.target.value) })
              }
              min={60}
              max={86400}
            />
            <p className="text-xs text-muted-foreground">{t("settings.seconds")}</p>
          </div>
        </div>

        <Button
          onClick={() => saveCategory("session", session as unknown as Record<string, unknown>)}
          disabled={saving === "session"}
          size="sm"
        >
          {saving === "session" ? <Spinner size="sm" /> : <Save className="h-4 w-4" />}
          {saved === "session" ? t("settings.saved") : t("settings.save")}
        </Button>
      </section>

      {/* Password complexity */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <Lock className="h-5 w-5 text-muted-foreground" />
          <div>
            <h2 className="text-lg font-semibold">{t("settings.passwordTitle")}</h2>
            <p className="text-sm text-muted-foreground">{t("settings.passwordDescription")}</p>
          </div>
        </div>

        <div className="space-y-3">
          <div className="space-y-2">
            <Label>{t("settings.minLength")}</Label>
            <Input
              type="number"
              value={passwordRules.minLength}
              onChange={(e) =>
                setPasswordRules({ ...passwordRules, minLength: Number(e.target.value) })
              }
              min={6}
              max={128}
            />
          </div>

          <ToggleOption
            label={t("settings.requireUppercase")}
            checked={passwordRules.requireUppercase}
            onChange={(v) =>
              setPasswordRules({ ...passwordRules, requireUppercase: v })
            }
          />
          <ToggleOption
            label={t("settings.requireLowercase")}
            checked={passwordRules.requireLowercase}
            onChange={(v) =>
              setPasswordRules({ ...passwordRules, requireLowercase: v })
            }
          />
          <ToggleOption
            label={t("settings.requireDigits")}
            checked={passwordRules.requireDigits}
            onChange={(v) =>
              setPasswordRules({ ...passwordRules, requireDigits: v })
            }
          />
          <ToggleOption
            label={t("settings.requireSpecial")}
            checked={passwordRules.requireSpecial}
            onChange={(v) =>
              setPasswordRules({ ...passwordRules, requireSpecial: v })
            }
          />
        </div>

        <Button
          onClick={() =>
            saveCategory(
              "auth_password_complexity",
              passwordRules as unknown as Record<string, unknown>,
            )
          }
          disabled={saving === "auth_password_complexity"}
          size="sm"
        >
          {saving === "auth_password_complexity" ? (
            <Spinner size="sm" />
          ) : (
            <Save className="h-4 w-4" />
          )}
          {saved === "auth_password_complexity"
            ? t("settings.saved")
            : t("settings.save")}
        </Button>
      </section>

      {/* Lockout settings */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-muted-foreground" />
            <div>
              <h2 className="text-lg font-semibold">{t("settings.lockoutTitle")}</h2>
              <p className="text-sm text-muted-foreground">{t("settings.lockoutDescription")}</p>
            </div>
          </div>
          <label className="relative inline-flex items-center cursor-pointer shrink-0">
            <input
              type="checkbox"
              checked={lockout.enabled}
              onChange={(e) => setLockout({ ...lockout, enabled: e.target.checked })}
              className="sr-only peer"
            />
            <div className="w-9 h-5 bg-muted rounded-full peer peer-checked:bg-primary transition-colors after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-background after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full" />
          </label>
        </div>

        <div className={`grid grid-cols-1 sm:grid-cols-3 gap-4 transition-opacity ${!lockout.enabled ? "opacity-50 pointer-events-none" : ""}`}>
          <div className="space-y-2">
            <Label className="whitespace-nowrap">{t("settings.maxAttempts")}</Label>
            <Input
              type="number"
              value={lockout.maxAttempts}
              onChange={(e) =>
                setLockout({ ...lockout, maxAttempts: Number(e.target.value) })
              }
              min={3}
              max={100}
              disabled={!lockout.enabled}
            />
          </div>
          <div className="space-y-2">
            <Label className="whitespace-nowrap">{t("settings.windowSeconds")}</Label>
            <Input
              type="number"
              value={lockout.windowSeconds}
              onChange={(e) =>
                setLockout({ ...lockout, windowSeconds: Number(e.target.value) })
              }
              min={30}
              max={3600}
              disabled={!lockout.enabled}
            />
          </div>
          <div className="space-y-2">
            <Label className="whitespace-nowrap">{t("settings.lockoutDuration")}</Label>
            <Input
              type="number"
              value={lockout.lockoutSeconds}
              onChange={(e) =>
                setLockout({ ...lockout, lockoutSeconds: Number(e.target.value) })
              }
              min={10}
              max={86400}
              disabled={!lockout.enabled}
            />
          </div>
        </div>

        <Button
          onClick={() =>
            saveCategory("user_lockout", lockout as unknown as Record<string, unknown>)
          }
          disabled={saving === "user_lockout"}
          size="sm"
        >
          {saving === "user_lockout" ? <Spinner size="sm" /> : <Save className="h-4 w-4" />}
          {saved === "user_lockout" ? t("settings.saved") : t("settings.save")}
        </Button>
      </section>

      {error && <p className="text-sm text-destructive mt-4">{error}</p>}
    </div>
  );
}

function ToggleOption({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex items-center gap-3 cursor-pointer">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="h-4 w-4 rounded border-input accent-primary"
      />
      <span className="text-sm">{label}</span>
    </label>
  );
}
