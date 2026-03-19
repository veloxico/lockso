import { api } from "./client";

export interface TotpSetupResponse {
  secret: string;
  otpauthUri: string;
  recoveryCodes: string[];
}

export interface TotpStatus {
  isEnabled: boolean;
  recoveryCodesRemaining: number;
}

export const totpApi = {
  getStatus: () => api.get<TotpStatus>("/2fa/status"),

  setup: () => api.post<TotpSetupResponse>("/2fa/setup", {}),

  enable: (secret: string, code: string, recoveryCodes: string[]) =>
    api.post<TotpStatus>("/2fa/enable", { secret, code, recoveryCodes }),

  verify: (code: string) =>
    api.post<{ verified: boolean }>("/2fa/verify", { code }),

  disable: (code: string) =>
    api.post<TotpStatus>("/2fa/disable", { code }),
};
