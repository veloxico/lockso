import { api } from "./client";
import type {
  VaultListItem,
  VaultView,
  CreateVaultRequest,
  UpdateVaultRequest,
  FolderView,
  FolderTreeNode,
  CreateFolderRequest,
  ItemListEntry,
  ItemView,
  CreateItemRequest,
  UpdateItemRequest,
  SnapshotListEntry,
  SnapshotView,
  SearchRequest,
  FavoriteResponse,
  TrashListEntry,
  TrashCountResponse,
  HealthReport,
  SendListEntry,
  CreateSendRequest,
  CreateSendResponse,
  SendPublicMeta,
  SendAccessView,
} from "@/types/vault";

// ─── Vaults ───

export const vaultApi = {
  list: () => api.get<VaultListItem[]>("/vaults"),

  get: (id: string) => api.get<VaultView>(`/vaults/${id}`),

  create: (data: CreateVaultRequest) => api.post<VaultView>("/vaults", data),

  update: (id: string, data: UpdateVaultRequest) =>
    api.put<VaultView>(`/vaults/${id}`, data),

  delete: (id: string) => api.delete<void>(`/vaults/${id}`),
};

// ─── Folders ───

export const folderApi = {
  list: (vaultId: string) =>
    api.get<FolderView[]>(`/folders?vaultId=${vaultId}`),

  tree: (vaultId: string) =>
    api.get<FolderTreeNode[]>(`/folders/tree?vaultId=${vaultId}`),

  create: (data: CreateFolderRequest) => api.post<FolderView>("/folders", data),

  update: (id: string, data: { name?: string; position?: number }) =>
    api.put<FolderView>(`/folders/${id}`, data),

  delete: (id: string) => api.delete<void>(`/folders/${id}`),

  move: (id: string, parentFolderId: string | null) =>
    api.post<FolderView>(`/folders/${id}/move`, { parentFolderId }),
};

// ─── Items ───

export const itemApi = {
  list: (vaultId: string, folderId?: string) => {
    let url = `/items?vaultId=${vaultId}`;
    if (folderId) url += `&folderId=${folderId}`;
    return api.get<ItemListEntry[]>(url);
  },

  get: (id: string) => api.get<ItemView>(`/items/${id}`),

  create: (data: CreateItemRequest) => api.post<ItemView>("/items", data),

  update: (id: string, data: UpdateItemRequest) =>
    api.put<ItemView>(`/items/${id}`, data),

  delete: (id: string) => api.delete<void>(`/items/${id}`),

  move: (id: string, data: { folderId?: string; vaultId?: string }) =>
    api.post<ItemView>(`/items/${id}/move`, data),

  search: (data: SearchRequest) =>
    api.post<ItemListEntry[]>("/items/search", data),

  recent: () => api.get<ItemListEntry[]>("/items/recent"),

  toggleFavorite: (id: string) =>
    api.post<FavoriteResponse>(`/items/${id}/favorite`),

  // Snapshots
  listSnapshots: (itemId: string) =>
    api.get<SnapshotListEntry[]>(`/items/${itemId}/snapshots`),

  getSnapshot: (itemId: string, snapshotId: string) =>
    api.get<SnapshotView>(`/items/${itemId}/snapshots/${snapshotId}`),

  revertToSnapshot: (itemId: string, snapshotId: string) =>
    api.post<ItemView>(`/items/${itemId}/revert-to-snapshot`, { snapshotId }),
};

// ─── Trash ───

export const trashApi = {
  list: () => api.get<TrashListEntry[]>("/trash"),

  count: () => api.get<TrashCountResponse>("/trash/count"),

  restore: (id: string) => api.post<void>(`/trash/${id}/restore`),

  permanentDelete: (id: string) => api.delete<void>(`/trash/${id}`),

  empty: () => api.delete<{ deleted: number }>("/trash"),
};

// ─── Health Report ───

export const healthApi = {
  report: () => api.get<HealthReport>("/health-report"),
};

// ─── Sends ───

export const sendApi = {
  list: () => api.get<SendListEntry[]>("/sends"),

  create: (data: CreateSendRequest) =>
    api.post<CreateSendResponse>("/sends", data),

  delete: (id: string) => api.delete<void>(`/sends/${id}`),
};

// Public send endpoints (no auth headers via fetch)
const BASE_URL = "/v1";

export const publicSendApi = {
  meta: async (accessId: string): Promise<SendPublicMeta> => {
    const resp = await fetch(`${BASE_URL}/public/sends/${accessId}`);
    if (!resp.ok) throw new Error("not_found");
    return resp.json();
  },

  access: async (
    accessId: string,
    passphrase?: string,
  ): Promise<SendAccessView> => {
    const resp = await fetch(`${BASE_URL}/public/sends/${accessId}/access`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(passphrase ? { passphrase } : {}),
    });
    if (!resp.ok) {
      const data = await resp.json().catch(() => ({}));
      throw new Error(data.code || "error");
    }
    return resp.json();
  },
};
