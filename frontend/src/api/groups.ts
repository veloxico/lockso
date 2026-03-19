import { api } from "./client";
import type {
  UserGroupListItem,
  UserGroupView,
  GroupMember,
} from "@/types/group";

interface UserGroup {
  id: string;
  name: string;
  description: string;
  creatorId: string | null;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export const groupApi = {
  /** List all groups */
  list: () => api.get<UserGroupListItem[]>("/groups"),

  /** Get a group with its members */
  get: (id: string) => api.get<UserGroupView>(`/groups/${id}`),

  /** Create a new group */
  create: (data: { name: string; description?: string }) =>
    api.post<UserGroup>("/groups", data),

  /** Update a group */
  update: (id: string, data: { name?: string; description?: string }) =>
    api.put<UserGroup>(`/groups/${id}`, data),

  /** Delete a group */
  delete: (id: string) => api.delete<void>(`/groups/${id}`),

  /** Add a member to a group */
  addMember: (groupId: string, userId: string) =>
    api.post<GroupMember>(`/groups/${groupId}/members`, { userId }),

  /** Remove a member from a group */
  removeMember: (groupId: string, userId: string) =>
    api.delete<void>(`/groups/${groupId}/members/${userId}`),
};
