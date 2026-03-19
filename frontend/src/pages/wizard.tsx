import { type FormEvent, useState, useEffect } from "react";
import { useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import {
  Eye,
  EyeOff,
  Check,
  X,
  ChevronRight,
  ChevronLeft,
  Server,
  UserPlus,
  ShieldCheck,
} from "lucide-react";
import { AuthLayout } from "@/components/layout/auth-layout";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Spinner } from "@/components/ui/spinner";
import { useHealthCheck, invalidateHealthCheck } from "@/hooks/use-health-check";
import { api } from "@/api/client";
import { toApiError } from "@/lib/api-error";
import type { UserView } from "@/types/api";

type WizardStep = "check" | "account" | "done";

interface StepIndicator {
  key: WizardStep;
  icon: typeof Server;
  labelKey: string;
}

const steps: StepIndicator[] = [
  { key: "check", icon: Server, labelKey: "wizard.stepCheck" },
  { key: "account", icon: UserPlus, labelKey: "wizard.stepAccount" },
  { key: "done", icon: ShieldCheck, labelKey: "wizard.stepDone" },
];

function getPasswordChecks(
  password: string,
  t: (k: string) => string,
) {
  return [
    { label: t("register.pwMinLength"), valid: password.length >= 8 },
    { label: t("register.pwUppercase"), valid: /[A-Z]/.test(password) },
    { label: t("register.pwLowercase"), valid: /[a-z]/.test(password) },
    { label: t("register.pwDigit"), valid: /\d/.test(password) },
    { label: t("register.pwSpecial"), valid: /[^a-zA-Z0-9]/.test(password) },
  ];
}

export function WizardPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { data: health } = useHealthCheck();

  const [step, setStep] = useState<WizardStep>("check");

  // Account form
  const [login, setLogin] = useState("");
  const [email, setEmail] = useState("");
  const [fullName, setFullName] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const passwordChecks = getPasswordChecks(password, t);
  const allPasswordChecksPass = passwordChecks.every((c) => c.valid);
  const passwordsMatch = password === confirmPassword && password.length > 0;

  // If system is already bootstrapped, redirect
  useEffect(() => {
    if (health?.isBootstrapped) {
      navigate("/login", { replace: true });
    }
  }, [health, navigate]);

  const currentStepIndex = steps.findIndex((s) => s.key === step);

  const handleCreateAccount = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!login.trim() || login.trim().length < 2) {
      setError(t("register.errorLoginTooShort"));
      return;
    }

    if (!/^[a-zA-Z0-9_\-.]+$/.test(login.trim())) {
      setError(t("register.errorLoginFormat"));
      return;
    }

    if (!allPasswordChecksPass) {
      setError(t("register.errorPasswordWeak"));
      return;
    }

    if (password !== confirmPassword) {
      setError(t("register.errorPasswordMismatch"));
      return;
    }

    if (
      email &&
      (!email.includes("@") || email.startsWith("@") || email.endsWith("@"))
    ) {
      setError(t("register.errorEmailInvalid"));
      return;
    }

    setIsSubmitting(true);

    try {
      await api.post<UserView>("/users/register", {
        login: login.trim(),
        password,
        email: email.trim() || undefined,
        fullName: fullName.trim() || undefined,
      });

      setStep("done");
    } catch (err) {
      const apiErr = toApiError(err);

      switch (apiErr.code) {
        case "LOGIN_ALREADY_TAKEN":
          setError(t("register.errorLoginTaken"));
          break;
        case "EMAIL_ALREADY_TAKEN":
          setError(t("register.errorEmailTaken"));
          break;
        case "PASSWORD_COMPLEXITY_FAILED":
          setError(t("register.errorPasswordWeak"));
          break;
        case "VALIDATION_ERROR":
          setError(apiErr.message);
          break;
        case "NETWORK_ERROR":
          setError(t("register.errorNetwork"));
          break;
        default:
          setError(apiErr.message || t("register.errorGeneric"));
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <AuthLayout>
      {/* Step indicators */}
      <div className="mb-6 flex items-center justify-center gap-2">
        {steps.map((s, i) => {
          const isActive = i === currentStepIndex;
          const isDone = i < currentStepIndex;
          return (
            <div key={s.key} className="flex items-center gap-2">
              {i > 0 && (
                <div
                  className={`h-px w-8 ${
                    isDone ? "bg-primary" : "bg-border"
                  }`}
                />
              )}
              <div
                className={`flex h-8 w-8 items-center justify-center rounded-full text-xs font-medium transition-colors ${
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : isDone
                      ? "bg-primary/20 text-primary"
                      : "bg-muted text-muted-foreground"
                }`}
              >
                {isDone ? (
                  <Check className="h-4 w-4" />
                ) : (
                  <s.icon className="h-4 w-4" />
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* Step: System check */}
      {step === "check" && (
        <Card>
          <CardHeader className="text-center">
            <CardTitle>{t("wizard.checkTitle")}</CardTitle>
            <CardDescription>{t("wizard.checkSubtitle")}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {health ? (
                <>
                  <ServiceRow
                    label={t("wizard.serviceDatabase")}
                    ok={health.database.status === "ok"}
                    t={t}
                  />
                  <ServiceRow
                    label={t("wizard.serviceRedis")}
                    ok={health.redis.status === "ok"}
                    t={t}
                  />
                  <ServiceRow
                    label={t("wizard.serviceStorage")}
                    ok={health.storage.status === "ok"}
                    t={t}
                  />

                  {health.status === "healthy" ? (
                    <div className="pt-4">
                      <Alert variant="success">
                        <AlertDescription>
                          {t("wizard.allServicesOk")}
                        </AlertDescription>
                      </Alert>
                      <Button
                        className="mt-4 w-full"
                        onClick={() => setStep("account")}
                      >
                        {t("wizard.next")}
                        <ChevronRight className="h-4 w-4" />
                      </Button>
                    </div>
                  ) : (
                    <div className="pt-4">
                      <Alert variant="destructive">
                        <AlertDescription>
                          {t("wizard.servicesNotReady")}
                        </AlertDescription>
                      </Alert>
                      <Button
                        variant="outline"
                        className="mt-4 w-full"
                        onClick={() => window.location.reload()}
                      >
                        {t("loading.retry")}
                      </Button>
                    </div>
                  )}
                </>
              ) : (
                <div className="flex justify-center py-8">
                  <Spinner size="lg" />
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Step: Create owner account */}
      {step === "account" && (
        <Card>
          <CardHeader className="text-center">
            <CardTitle>{t("wizard.accountTitle")}</CardTitle>
            <CardDescription>{t("wizard.accountSubtitle")}</CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleCreateAccount} className="space-y-4">
              {error && (
                <Alert variant="destructive">
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}

              <div className="space-y-2">
                <Label htmlFor="login">{t("register.loginLabel")}</Label>
                <Input
                  id="login"
                  value={login}
                  onChange={(e) => setLogin(e.target.value)}
                  placeholder={t("register.loginPlaceholder")}
                  autoComplete="username"
                  autoFocus
                  disabled={isSubmitting}
                  maxLength={100}
                />
                <p className="text-xs text-muted-foreground">
                  {t("register.loginHint")}
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="fullName">
                  {t("register.fullNameLabel")}
                  <span className="ml-1 font-normal text-muted-foreground">
                    ({t("common.optional")})
                  </span>
                </Label>
                <Input
                  id="fullName"
                  value={fullName}
                  onChange={(e) => setFullName(e.target.value)}
                  placeholder={t("register.fullNamePlaceholder")}
                  autoComplete="name"
                  disabled={isSubmitting}
                  maxLength={255}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="email">
                  {t("register.emailLabel")}
                  <span className="ml-1 font-normal text-muted-foreground">
                    ({t("common.optional")})
                  </span>
                </Label>
                <Input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder={t("register.emailPlaceholder")}
                  autoComplete="email"
                  disabled={isSubmitting}
                  maxLength={255}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password">
                  {t("register.passwordLabel")}
                </Label>
                <div className="relative">
                  <Input
                    id="password"
                    type={showPassword ? "text" : "password"}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder={t("register.passwordPlaceholder")}
                    autoComplete="new-password"
                    disabled={isSubmitting}
                    maxLength={256}
                    className="pr-10"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword((v) => !v)}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                    tabIndex={-1}
                  >
                    {showPassword ? (
                      <EyeOff className="h-4 w-4" />
                    ) : (
                      <Eye className="h-4 w-4" />
                    )}
                  </button>
                </div>

                {password.length > 0 && (
                  <div className="mt-2 space-y-1">
                    {passwordChecks.map((check) => (
                      <div
                        key={check.label}
                        className="flex items-center gap-2 text-xs"
                      >
                        {check.valid ? (
                          <Check className="h-3 w-3 text-success" />
                        ) : (
                          <X className="h-3 w-3 text-muted-foreground" />
                        )}
                        <span
                          className={
                            check.valid
                              ? "text-success"
                              : "text-muted-foreground"
                          }
                        >
                          {check.label}
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="confirmPassword">
                  {t("register.confirmPasswordLabel")}
                </Label>
                <Input
                  id="confirmPassword"
                  type={showPassword ? "text" : "password"}
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  placeholder={t("register.confirmPasswordPlaceholder")}
                  autoComplete="new-password"
                  disabled={isSubmitting}
                  maxLength={256}
                />
                {confirmPassword.length > 0 && !passwordsMatch && (
                  <p className="text-xs text-destructive">
                    {t("register.errorPasswordMismatch")}
                  </p>
                )}
              </div>

              <div className="flex gap-3 pt-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setStep("check")}
                  disabled={isSubmitting}
                >
                  <ChevronLeft className="h-4 w-4" />
                  {t("wizard.back")}
                </Button>
                <Button
                  type="submit"
                  className="flex-1"
                  disabled={
                    isSubmitting ||
                    !allPasswordChecksPass ||
                    !passwordsMatch
                  }
                >
                  {isSubmitting ? (
                    <>
                      <Spinner size="sm" />
                      {t("wizard.creating")}
                    </>
                  ) : (
                    <>
                      {t("wizard.createOwner")}
                      <ChevronRight className="h-4 w-4" />
                    </>
                  )}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      )}

      {/* Step: Done */}
      {step === "done" && (
        <Card>
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-success/10">
              <ShieldCheck className="h-8 w-8 text-success" />
            </div>
            <CardTitle>{t("wizard.doneTitle")}</CardTitle>
            <CardDescription>{t("wizard.doneSubtitle")}</CardDescription>
          </CardHeader>
          <CardContent>
            <Button
              className="w-full"
              onClick={() => {
                invalidateHealthCheck();
                navigate("/login", { replace: true });
              }}
            >
              {t("wizard.goToLogin")}
            </Button>
          </CardContent>
        </Card>
      )}
    </AuthLayout>
  );
}

function ServiceRow({
  label,
  ok,
  t,
}: {
  label: string;
  ok: boolean;
  t: (k: string) => string;
}) {
  return (
    <div className="flex items-center justify-between rounded-md border border-border px-4 py-3">
      <span className="text-sm font-medium">{label}</span>
      {ok ? (
        <div className="flex items-center gap-1.5 text-sm text-success">
          <Check className="h-4 w-4" />
          <span>{t("wizard.statusOk")}</span>
        </div>
      ) : (
        <div className="flex items-center gap-1.5 text-sm text-destructive">
          <X className="h-4 w-4" />
          <span>{t("wizard.statusError")}</span>
        </div>
      )}
    </div>
  );
}
