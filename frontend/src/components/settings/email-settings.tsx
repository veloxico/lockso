import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Mail, Send, Check, AlertCircle, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { emailApi, type UpdateEmailSettings } from "@/api/email";

const PROVIDERS = [
  { value: "smtp", label: "SMTP", docsUrl: "" },
  { value: "sendgrid", label: "SendGrid", docsUrl: "https://docs.sendgrid.com/for-developers/sending-email/api-getting-started" },
  { value: "ses", label: "Amazon SES", docsUrl: "https://docs.aws.amazon.com/ses/latest/dg/send-email-api.html" },
  { value: "resend", label: "Resend", docsUrl: "https://resend.com/docs/introduction" },
  { value: "mailgun", label: "Mailgun", docsUrl: "https://documentation.mailgun.com/docs/mailgun/quickstart-guide/how-to-start-sending-email/" },
  { value: "postmark", label: "Postmark", docsUrl: "https://postmarkapp.com/developer/api/overview" },
  { value: "mandrill", label: "Mandrill", docsUrl: "https://mailchimp.com/developer/transactional/docs/fundamentals/" },
];

type ProviderConfig = Record<string, string | number | boolean>;

function getDefaultConfig(provider: string): ProviderConfig {
  switch (provider) {
    case "smtp":
      return { host: "", port: 587, username: "", password: "", useTls: false };
    case "ses":
      return { accessKeyId: "", secretAccessKey: "", region: "us-east-1" };
    case "mailgun":
      return { apiKey: "", domain: "", euRegion: false };
    default:
      // sendgrid, resend, postmark, mandrill
      return { apiKey: "" };
  }
}

function getConfigFields(provider: string): ConfigFieldDef[] {
  switch (provider) {
    case "smtp":
      return [
        { key: "host", label: "emailSettings.fields.host", type: "text" },
        { key: "port", label: "emailSettings.fields.port", type: "number" },
        { key: "username", label: "emailSettings.fields.username", type: "text" },
        { key: "password", label: "emailSettings.fields.password", type: "password" },
        { key: "useTls", label: "emailSettings.fields.useTls", type: "toggle" },
      ];
    case "ses":
      return [
        { key: "accessKeyId", label: "emailSettings.fields.accessKeyId", type: "text" },
        { key: "secretAccessKey", label: "emailSettings.fields.secretAccessKey", type: "password" },
        { key: "region", label: "emailSettings.fields.region", type: "text" },
      ];
    case "mailgun":
      return [
        { key: "apiKey", label: "emailSettings.fields.apiKey", type: "password" },
        { key: "domain", label: "emailSettings.fields.domain", type: "text" },
        { key: "euRegion", label: "emailSettings.fields.euRegion", type: "toggle" },
      ];
    default:
      return [
        { key: "apiKey", label: "emailSettings.fields.apiKey", type: "password" },
      ];
  }
}

interface ConfigFieldDef {
  key: string;
  label: string;
  type: "text" | "password" | "number" | "toggle";
}

export function EmailSettings() {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [testEmail, setTestEmail] = useState("");

  const [provider, setProvider] = useState("smtp");
  const [isEnabled, setIsEnabled] = useState(false);
  const [fromName, setFromName] = useState("Lockso");
  const [fromEmail, setFromEmail] = useState("");
  const [config, setConfig] = useState<ProviderConfig>(getDefaultConfig("smtp"));

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await emailApi.get();
      if (data) {
        setProvider(data.provider);
        setIsEnabled(data.isEnabled);
        setFromName(data.fromName);
        setFromEmail(data.fromEmail);
        setConfig(data.config as ProviderConfig);
      }
    } catch {
      setError(t("settings.errorLoadFailed"));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    load();
  }, [load]);

  const handleProviderChange = (newProvider: string) => {
    setProvider(newProvider);
    setConfig(getDefaultConfig(newProvider));
  };

  const handleSave = async () => {
    setSaving(true);
    setError("");
    setSuccess("");
    try {
      const payload: UpdateEmailSettings = {
        provider,
        isEnabled,
        fromName,
        fromEmail,
        config,
      };
      await emailApi.update(payload);
      setSuccess(t("settings.saved"));
      setTimeout(() => setSuccess(""), 3000);
    } catch {
      setError(t("settings.errorSaveFailed"));
    } finally {
      setSaving(false);
    }
  };

  const handleTest = async () => {
    if (!testEmail.trim()) return;
    setTesting(true);
    setError("");
    setSuccess("");
    try {
      await emailApi.sendTest(testEmail.trim());
      setSuccess(t("emailSettings.testSent"));
      setTimeout(() => setSuccess(""), 3000);
    } catch {
      setError(t("emailSettings.testFailed"));
    } finally {
      setTesting(false);
    }
  };

  const updateConfig = (key: string, value: string | number | boolean) => {
    setConfig((prev) => ({ ...prev, [key]: value }));
  };

  if (loading) {
    return (
      <div className="flex justify-center py-12">
        <Spinner size="md" />
      </div>
    );
  }

  const fields = getConfigFields(provider);

  return (
    <div className="max-w-2xl space-y-6">
      <div className="flex items-center gap-2">
        <Mail className="h-5 w-5 text-muted-foreground" />
        <div>
          <h2 className="text-lg font-semibold">{t("emailSettings.title")}</h2>
          <p className="text-sm text-muted-foreground">
            {t("emailSettings.description")}
          </p>
        </div>
      </div>

      {error && (
        <div className="flex items-center gap-2 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          <AlertCircle className="h-4 w-4 shrink-0" />
          {error}
        </div>
      )}
      {success && (
        <div className="flex items-center gap-2 rounded-md border border-success/30 bg-success/10 px-3 py-2 text-sm text-success">
          <Check className="h-4 w-4 shrink-0" />
          {success}
        </div>
      )}

      {/* Enable toggle */}
      <label className="flex items-center gap-3 cursor-pointer">
        <input
          type="checkbox"
          checked={isEnabled}
          onChange={(e) => setIsEnabled(e.target.checked)}
          className="h-4 w-4 rounded border-input accent-primary"
        />
        <span className="text-sm font-medium">{t("emailSettings.enabled")}</span>
      </label>

      {/* Provider selection */}
      <div className="space-y-2">
        <Label>{t("emailSettings.provider")}</Label>
        <select
          value={provider}
          onChange={(e) => handleProviderChange(e.target.value)}
          className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
        >
          {PROVIDERS.map((p) => (
            <option key={p.value} value={p.value}>
              {p.label}
            </option>
          ))}
        </select>
        {(() => {
          const current = PROVIDERS.find((p) => p.value === provider);
          if (!current?.docsUrl) return null;
          return (
            <a
              href={current.docsUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 text-xs text-primary hover:underline"
            >
              <ExternalLink className="h-3 w-3" />
              {t("emailSettings.providerDocs", { provider: current.label })}
            </a>
          );
        })()}
      </div>

      {/* From fields */}
      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-2">
          <Label>{t("emailSettings.fromName")}</Label>
          <Input
            value={fromName}
            onChange={(e) => setFromName(e.target.value)}
            placeholder="Lockso"
          />
        </div>
        <div className="space-y-2">
          <Label>{t("emailSettings.fromEmail")}</Label>
          <Input
            type="email"
            value={fromEmail}
            onChange={(e) => setFromEmail(e.target.value)}
            placeholder="noreply@example.com"
          />
        </div>
      </div>

      {/* Provider config fields */}
      <div className="space-y-4 rounded-lg border border-border p-4">
        <h3 className="text-sm font-medium">
          {PROVIDERS.find((p) => p.value === provider)?.label}{" "}
          {t("emailSettings.configuration")}
        </h3>

        {fields.map((field) => (
          <div key={field.key}>
            {field.type === "toggle" ? (
              <label className="flex items-center gap-2 cursor-pointer text-sm">
                <input
                  type="checkbox"
                  checked={!!config[field.key]}
                  onChange={(e) => updateConfig(field.key, e.target.checked)}
                  className="h-4 w-4 rounded border-input accent-primary"
                />
                {t(field.label)}
              </label>
            ) : (
              <div className="space-y-1">
                <Label className="text-xs">{t(field.label)}</Label>
                <Input
                  type={field.type}
                  value={String(config[field.key] ?? "")}
                  onChange={(e) =>
                    updateConfig(
                      field.key,
                      field.type === "number"
                        ? Number(e.target.value)
                        : e.target.value,
                    )
                  }
                />
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Save */}
      <Button onClick={handleSave} disabled={saving}>
        {saving ? <Spinner size="sm" /> : t("settings.save")}
      </Button>

      {/* Test email */}
      <div className="space-y-3 rounded-lg border border-border p-4">
        <h3 className="text-sm font-medium">{t("emailSettings.testTitle")}</h3>
        <div className="flex gap-2">
          <Input
            type="email"
            value={testEmail}
            onChange={(e) => setTestEmail(e.target.value)}
            placeholder={t("emailSettings.testPlaceholder")}
            className="flex-1"
          />
          <Button
            variant="outline"
            onClick={handleTest}
            disabled={testing || !testEmail.trim()}
          >
            {testing ? (
              <Spinner size="sm" />
            ) : (
              <>
                <Send className="mr-1.5 h-3.5 w-3.5" />
                {t("emailSettings.testSend")}
              </>
            )}
          </Button>
        </div>
      </div>
    </div>
  );
}
