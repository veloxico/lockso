import { type FormEvent, useState } from "react";
import { Link, useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import { Eye, EyeOff, Check, X } from "lucide-react";
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
import { api } from "@/api/client";
import { toApiError } from "@/lib/api-error";
import type { UserView } from "@/types/api";

/** Client-side password strength indicators (backend has final say). */
interface PasswordCheck {
  label: string;
  valid: boolean;
}

function getPasswordChecks(
  password: string,
  t: (k: string) => string,
): PasswordCheck[] {
  return [
    { label: t("register.pwMinLength"), valid: password.length >= 8 },
    { label: t("register.pwUppercase"), valid: /[A-Z]/.test(password) },
    { label: t("register.pwLowercase"), valid: /[a-z]/.test(password) },
    { label: t("register.pwDigit"), valid: /\d/.test(password) },
    { label: t("register.pwSpecial"), valid: /[^a-zA-Z0-9]/.test(password) },
  ];
}

export function RegisterPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

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

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);

    // Client-side validation
    if (!login.trim()) {
      setError(t("register.errorLoginRequired"));
      return;
    }

    if (login.trim().length < 2) {
      setError(t("register.errorLoginTooShort"));
      return;
    }

    if (!/^[a-zA-Z0-9_\-.]+$/.test(login.trim())) {
      setError(t("register.errorLoginFormat"));
      return;
    }

    if (!password) {
      setError(t("register.errorPasswordRequired"));
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

    if (email && (!email.includes("@") || email.startsWith("@") || email.endsWith("@"))) {
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

      // Registration successful — redirect to login
      navigate("/login", {
        replace: true,
        state: { registered: true },
      });
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
        case "TOO_MANY_REQUESTS":
          setError(t("register.errorTooManyAttempts"));
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
      <Card>
        <CardHeader className="text-center">
          <CardTitle>{t("register.title")}</CardTitle>
          <CardDescription>{t("register.subtitle")}</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <Label htmlFor="login">{t("register.loginLabel")}</Label>
              <Input
                id="login"
                type="text"
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
                type="text"
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
              <Label htmlFor="password">{t("register.passwordLabel")}</Label>
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

              {/* Password strength indicators */}
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

            <Button
              type="submit"
              className="w-full"
              disabled={
                isSubmitting || !allPasswordChecksPass || !passwordsMatch
              }
            >
              {isSubmitting ? (
                <>
                  <Spinner size="sm" />
                  {t("register.creating")}
                </>
              ) : (
                t("register.submit")
              )}
            </Button>
          </form>

          <div className="mt-6 text-center text-sm text-muted-foreground">
            {t("register.hasAccount")}{" "}
            <Link
              to="/login"
              className="font-medium text-primary hover:underline"
            >
              {t("register.signIn")}
            </Link>
          </div>
        </CardContent>
      </Card>
    </AuthLayout>
  );
}
