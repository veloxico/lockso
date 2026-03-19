import { BrowserRouter, Routes, Route, Navigate } from "react-router";
import { useAuthStore } from "@/stores/auth";
import { LoadingPage } from "@/pages/loading";
import { LoginPage } from "@/pages/login";
import { RegisterPage } from "@/pages/register";
import { WizardPage } from "@/pages/wizard";
import { TwoFactorPage } from "@/pages/two-factor";
import { VaultDetailPage } from "@/pages/vault-detail";
import { FavoritesPage } from "@/pages/favorites";
import { RecentPage } from "@/pages/recent";
import { SettingsPage } from "@/pages/settings";
import { TrashPage } from "@/pages/trash";
import { HealthPage } from "@/pages/health";
import { SendsPage } from "@/pages/sends";
import { SendViewPage } from "@/pages/send-view";
import { CommandPalette } from "@/components/command-palette";
import { useHealthCheck } from "@/hooks/use-health-check";
import { Spinner } from "@/components/ui/spinner";

/**
 * Route guard — checks bootstrap status first, then auth.
 */
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const isTwoFactorRequired = useAuthStore((s) => s.isTwoFactorAuthRequired);
  const { data, isLoading } = useHealthCheck();

  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <Spinner size="lg" />
      </div>
    );
  }

  if (data && !data.isBootstrapped) {
    return <Navigate to="/wizard" replace />;
  }

  if (isTwoFactorRequired) {
    return <Navigate to="/2fa" replace />;
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return (
    <>
      {children}
      <CommandPalette />
    </>
  );
}

/**
 * Guest route — checks bootstrap first, then redirects authenticated users.
 */
function GuestRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const { data, isLoading } = useHealthCheck();

  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <Spinner size="lg" />
      </div>
    );
  }

  if (data && !data.isBootstrapped) {
    return <Navigate to="/wizard" replace />;
  }

  if (isAuthenticated) {
    return <Navigate to="/" replace />;
  }

  return <>{children}</>;
}

/**
 * 2FA route guard.
 */
function TwoFactorRoute({ children }: { children: React.ReactNode }) {
  const isTwoFactorRequired = useAuthStore((s) => s.isTwoFactorAuthRequired);
  const accessToken = useAuthStore((s) => s.accessToken);

  if (!accessToken) {
    return <Navigate to="/login" replace />;
  }

  if (!isTwoFactorRequired) {
    return <Navigate to="/" replace />;
  }

  return <>{children}</>;
}

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        {/* Bootstrap */}
        <Route path="/loading" element={<LoadingPage />} />
        <Route path="/wizard" element={<WizardPage />} />

        {/* Guest */}
        <Route path="/login" element={<GuestRoute><LoginPage /></GuestRoute>} />
        <Route path="/register" element={<GuestRoute><RegisterPage /></GuestRoute>} />

        {/* 2FA */}
        <Route path="/2fa" element={<TwoFactorRoute><TwoFactorPage /></TwoFactorRoute>} />

        {/* App — all protected */}
        <Route path="/" element={<ProtectedRoute><Navigate to="/recent" replace /></ProtectedRoute>} />
        <Route path="/recent" element={<ProtectedRoute><RecentPage /></ProtectedRoute>} />
        <Route path="/favorites" element={<ProtectedRoute><FavoritesPage /></ProtectedRoute>} />
        <Route path="/trash" element={<ProtectedRoute><TrashPage /></ProtectedRoute>} />
        <Route path="/health" element={<ProtectedRoute><HealthPage /></ProtectedRoute>} />
        <Route path="/sends" element={<ProtectedRoute><SendsPage /></ProtectedRoute>} />
        <Route path="/vault/:vaultId" element={<ProtectedRoute><VaultDetailPage /></ProtectedRoute>} />
        <Route path="/settings" element={<ProtectedRoute><SettingsPage /></ProtectedRoute>} />

        {/* Public (no auth) */}
        <Route path="/send/:accessId" element={<SendViewPage />} />

        {/* Catch-all */}
        <Route path="*" element={<Navigate to="/loading" replace />} />
      </Routes>
    </BrowserRouter>
  );
}
