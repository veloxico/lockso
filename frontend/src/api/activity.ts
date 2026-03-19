import { api } from "./client";
import type { PaginatedActivityLogs, ActivityLogQuery } from "@/types/activity";

export const activityApi = {
  listGlobal: (params?: ActivityLogQuery) => {
    const qs = new URLSearchParams();
    if (params?.page) qs.set("page", String(params.page));
    if (params?.perPage) qs.set("perPage", String(params.perPage));
    if (params?.userId) qs.set("userId", params.userId);
    if (params?.action) qs.set("action", params.action);
    const q = qs.toString();
    return api.get<PaginatedActivityLogs>(`/activity${q ? `?${q}` : ""}`);
  },

  listVault: (vaultId: string, params?: ActivityLogQuery) => {
    const qs = new URLSearchParams();
    if (params?.page) qs.set("page", String(params.page));
    if (params?.perPage) qs.set("perPage", String(params.perPage));
    const q = qs.toString();
    return api.get<PaginatedActivityLogs>(
      `/vaults/${vaultId}/activity${q ? `?${q}` : ""}`,
    );
  },
};
