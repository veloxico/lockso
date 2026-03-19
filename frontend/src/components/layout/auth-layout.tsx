import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { LocksoLogo } from "@/components/lockso-logo";
import { LanguageToggle } from "@/components/language-toggle";
import { ThemeToggle } from "@/components/theme-toggle";

interface AuthLayoutProps {
  children: ReactNode;
}

/**
 * Layout for authentication pages (login, register, wizard).
 * Centered card with Lockso branding.
 */
export function AuthLayout({ children }: AuthLayoutProps) {
  const { t } = useTranslation();

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background px-4">
      {/* Language + theme toggles in top-right */}
      <div className="absolute right-4 top-4 flex items-center gap-2">
        <LanguageToggle />
        <ThemeToggle />
      </div>

      <div className="mb-8 text-center">
        <div className="flex justify-center">
          <LocksoLogo size="lg" />
        </div>
        <p className="mt-2 text-muted-foreground">
          {t("common.tagline")}
        </p>
      </div>
      <div className="w-full max-w-md">{children}</div>
    </div>
  );
}
