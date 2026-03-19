import { api } from "./client";
import type {
  AdminUserListItem,
  AppSettings,
  UserRoleView,
} from "@/types/admin";
import type { UserView } from "@/types/api";

// ─── Settings ───

export const settingsApi = {
  get: () => api.get<AppSettings>("/settings"),

  updateCategory: (category: string, value: Record<string, unknown>) =>
    api.put<AppSettings>(`/settings/${category}`, value),
};

// ─── User Management ───

export const userManagementApi = {
  list: () => api.get<AdminUserListItem[]>("/users"),

  create: (data: {
    login: string;
    password: string;
    email?: string;
    fullName?: string;
    roleId?: string;
  }) => api.post<UserView>("/users", data),

  listRoles: () => api.get<UserRoleView[]>("/users/roles"),

  updateRole: (userId: string, roleId: string) =>
    api.put<UserView>(`/users/${userId}/role`, { roleId }),

  setBlocked: (userId: string, isBlocked: boolean) =>
    api.put<UserView>(`/users/${userId}/block`, { isBlocked }),

  delete: (userId: string) => api.delete<void>(`/users/${userId}`),
};

// ─── Webhooks ───

export interface WebhookView {
  id: string;
  name: string;
  provider: string;
  urlMasked: string;
  events: string[];
  isEnabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateWebhookReq {
  name: string;
  provider: string;
  url: string;
  events: string[];
}

export const webhookApi = {
  list: () => api.get<WebhookView[]>("/webhooks"),
  create: (data: CreateWebhookReq) => api.post<WebhookView>("/webhooks", data),
  update: (id: string, data: Record<string, unknown>) =>
    api.put<WebhookView>(`/webhooks/${id}`, data),
  delete: (id: string) => api.delete<void>(`/webhooks/${id}`),
  test: (id: string) => api.post<void>(`/webhooks/${id}/test`, {}),
};

// ─── API Keys ───

export interface ApiKeyView {
  id: string;
  name: string;
  keyPrefix: string;
  userId: string;
  permission: string;
  vaultId: string | null;
  expiresAt: string | null;
  lastUsedAt: string | null;
  isEnabled: boolean;
  createdAt: string;
}

export interface ApiKeyCreated {
  id: string;
  name: string;
  key: string;
  keyPrefix: string;
  permission: string;
  vaultId: string | null;
  expiresAt: string | null;
  createdAt: string;
}

export const apiKeyApi = {
  list: () => api.get<ApiKeyView[]>("/api-keys"),
  create: (data: { name: string; permission: string; vaultId?: string; expiresAt?: string }) =>
    api.post<ApiKeyCreated>("/api-keys", data),
  delete: (id: string) => api.delete<void>(`/api-keys/${id}`),
};
