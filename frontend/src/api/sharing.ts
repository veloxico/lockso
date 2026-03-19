import { api } from "./client";
import type { VaultMember, ResourceAccessLevel } from "@/types/sharing";

export const sharingApi = {
  /** List available access levels */
  listAccessLevels: () =>
    api.get<ResourceAccessLevel[]>("/sharing/access-levels"),

  /** List members of a vault */
  listMembers: (vaultId: string) =>
    api.get<VaultMember[]>(`/sharing/${vaultId}/members`),

  /** Share a vault with a user */
  share: (vaultId: string, userId: string, resourceAccessId: string) =>
    api.post<VaultMember>(`/sharing/${vaultId}/members`, {
      userId,
      resourceAccessId,
    }),

  /** Update a member's access level */
  updateAccess: (
    vaultId: string,
    userId: string,
    resourceAccessId: string,
  ) =>
    api.put<VaultMember>(`/sharing/${vaultId}/members/${userId}`, {
      resourceAccessId,
    }),

  /** Revoke a user's access */
  revokeAccess: (vaultId: string, userId: string) =>
    api.delete<void>(`/sharing/${vaultId}/members/${userId}`),
};
