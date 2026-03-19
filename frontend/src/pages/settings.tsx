import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Settings, Shield, Users, UsersRound, KeyRound, ScrollText, Monitor, Mail, Fingerprint, Bell, Code } from "lucide-react";
import { AppLayout } from "@/components/layout/app-layout";
import { cn } from "@/lib/utils";
import { GeneralSettings } from "@/components/settings/general-settings";
import { SecuritySettings } from "@/components/settings/security-settings";
import { UserManagement } from "@/components/settings/user-management";
import { GroupManagement } from "@/components/settings/group-management";
import { TwoFactorSettings } from "@/components/settings/two-factor-settings";
import { ActivityLog } from "@/components/settings/activity-log";
import { SessionManagement } from "@/components/settings/session-management";
import { EmailSettings } from "@/components/settings/email-settings";
import { WebAuthnSettings } from "@/components/settings/webauthn-settings";
import { WebhookSettings } from "@/components/settings/webhook-settings";
import { ApiKeySettings } from "@/components/settings/api-key-settings";

type Tab = "general" | "security" | "2fa" | "webauthn" | "sessions" | "email" | "webhooks" | "apikeys" | "users" | "groups" | "activity";

const TABS: { id: Tab; icon: typeof Settings; labelKey: string }[] = [
  { id: "general", icon: Settings, labelKey: "settings.tabGeneral" },
  { id: "security", icon: Shield, labelKey: "settings.tabSecurity" },
  { id: "2fa", icon: KeyRound, labelKey: "settings.tab2FA" },
  { id: "webauthn", icon: Fingerprint, labelKey: "settings.tabWebAuthn" },
  { id: "sessions", icon: Monitor, labelKey: "settings.tabSessions" },
  { id: "email", icon: Mail, labelKey: "settings.tabEmail" },
  { id: "webhooks", icon: Bell, labelKey: "settings.tabWebhooks" },
  { id: "apikeys", icon: Code, labelKey: "settings.tabApiKeys" },
  { id: "users", icon: Users, labelKey: "settings.tabUsers" },
  { id: "groups", icon: UsersRound, labelKey: "settings.tabGroups" },
  { id: "activity", icon: ScrollText, labelKey: "settings.tabActivity" },
];

export function SettingsPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<Tab>("general");

  return (
    <AppLayout>
      <h1 className="text-2xl font-bold tracking-tight">{t("nav.settings")}</h1>

      {/* Tabs */}
      <div className="mt-4 flex gap-1 border-b border-border overflow-x-auto">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={cn(
              "flex items-center gap-2 px-4 py-2.5 text-sm font-medium transition-colors border-b-2 -mb-px whitespace-nowrap",
              activeTab === tab.id
                ? "border-primary text-primary"
                : "border-transparent text-muted-foreground hover:text-foreground hover:border-muted-foreground/30",
            )}
          >
            <tab.icon className="h-4 w-4" />
            {t(tab.labelKey)}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="mt-6">
        {activeTab === "general" && <GeneralSettings />}
        {activeTab === "security" && <SecuritySettings />}
        {activeTab === "2fa" && <TwoFactorSettings />}
        {activeTab === "webauthn" && <WebAuthnSettings />}
        {activeTab === "sessions" && <SessionManagement />}
        {activeTab === "email" && <EmailSettings />}
        {activeTab === "webhooks" && <WebhookSettings />}
        {activeTab === "apikeys" && <ApiKeySettings />}
        {activeTab === "users" && <UserManagement />}
        {activeTab === "groups" && <GroupManagement />}
        {activeTab === "activity" && <ActivityLog />}
      </div>
    </AppLayout>
  );
}
