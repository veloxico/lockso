import { type FormEvent, useState } from "react";
import { Link, useLocation, useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import { Eye, EyeOff, CheckCircle } from "lucide-react";
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
import { useAuthStore } from "@/stores/auth";
import { toApiError } from "@/lib/api-error";
import type { LoginResponse } from "@/types/api";

export function LoginPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const setAuth = useAuthStore((s) => s.setAuth);

  // Show success message if redirected from registration
  const justRegistered = (location.state as { registered?: boolean } | null)
    ?.registered;

  const [login, setLogin] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!login.trim() || !password) {
      setError(t("login.errorRequired"));
      return;
    }

    setIsSubmitting(true);

    try {
      const response = await api.post<LoginResponse>("/users/login", {
        login: login.trim(),
        password,
        clientType: "Web",
      });

      setAuth(response);

      // If 2FA or master key is required, navigate to the challenge page (Phase 6).
      // For now, navigate to dashboard — setAuth won't set isAuthenticated
      // if 2FA is required, so the ProtectedRoute guard will handle it.
      navigate("/", { replace: true });
    } catch (err) {
      const apiErr = toApiError(err);

      switch (apiErr.code) {
        case "INVALID_LOGIN_OR_PASSWORD":
          setError(t("login.errorInvalidCredentials"));
          break;
        case "TOO_MANY_REQUESTS":
          setError(t("login.errorTooManyAttempts"));
          break;
        case "NETWORK_ERROR":
          setError(t("login.errorNetwork"));
          break;
        default:
          setError(apiErr.message || t("login.errorGeneric"));
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <AuthLayout>
      <Card>
        <CardHeader className="text-center">
          <CardTitle>{t("login.title")}</CardTitle>
          <CardDescription>{t("login.subtitle")}</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            {justRegistered && !error && (
              <Alert variant="success">
                <CheckCircle className="h-4 w-4" />
                <AlertDescription>
                  {t("login.registrationSuccess")}
                </AlertDescription>
              </Alert>
            )}

            {error && (
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <Label htmlFor="login">{t("login.loginLabel")}</Label>
              <Input
                id="login"
                type="text"
                value={login}
                onChange={(e) => setLogin(e.target.value)}
                placeholder={t("login.loginPlaceholder")}
                autoComplete="username"
                autoFocus
                disabled={isSubmitting}
                maxLength={100}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="password">{t("login.passwordLabel")}</Label>
              <div className="relative">
                <Input
                  id="password"
                  type={showPassword ? "text" : "password"}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder={t("login.passwordPlaceholder")}
                  autoComplete="current-password"
                  disabled={isSubmitting}
                  maxLength={256}
                  className="pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword((v) => !v)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                  tabIndex={-1}
                  aria-label={
                    showPassword
                      ? t("login.hidePassword")
                      : t("login.showPassword")
                  }
                >
                  {showPassword ? (
                    <EyeOff className="h-4 w-4" />
                  ) : (
                    <Eye className="h-4 w-4" />
                  )}
                </button>
              </div>
            </div>

            <Button
              type="submit"
              className="w-full"
              disabled={isSubmitting}
            >
              {isSubmitting ? (
                <>
                  <Spinner size="sm" />
                  {t("login.signingIn")}
                </>
              ) : (
                t("login.submit")
              )}
            </Button>
          </form>

          <div className="mt-6 text-center text-sm text-muted-foreground">
            {t("login.noAccount")}{" "}
            <Link
              to="/register"
              className="font-medium text-primary hover:underline"
            >
              {t("login.createAccount")}
            </Link>
          </div>
        </CardContent>
      </Card>
    </AuthLayout>
  );
}
