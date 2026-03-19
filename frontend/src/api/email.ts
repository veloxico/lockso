import { api } from "./client";

export interface EmailSettingsView {
  provider: string;
  isEnabled: boolean;
  fromName: string;
  fromEmail: string;
  config: Record<string, unknown>;
  updatedAt: string;
}

export interface UpdateEmailSettings {
  provider: string;
  isEnabled: boolean;
  fromName: string;
  fromEmail: string;
  config: Record<string, unknown>;
}

export const emailApi = {
  get: () => api.get<EmailSettingsView | null>("/email"),

  update: (data: UpdateEmailSettings) =>
    api.put<EmailSettingsView>("/email", data),

  sendTest: (to: string) =>
    api.post<{ message: string }>("/email/test", { to }),
};
