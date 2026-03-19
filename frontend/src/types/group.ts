/** Group list item from GET /v1/groups */
export interface UserGroupListItem {
  id: string;
  name: string;
  description: string;
  memberCount: number;
  isActive: boolean;
  createdAt: string;
}

/** Full group view from GET /v1/groups/:id */
export interface UserGroupView {
  id: string;
  name: string;
  description: string;
  creatorId: string | null;
  isActive: boolean;
  members: GroupMember[];
  createdAt: string;
  updatedAt: string;
}

/** Group member */
export interface GroupMember {
  id: string;
  userId: string;
  login: string;
  fullName: string;
  email: string | null;
  addedBy: string | null;
  createdAt: string;
}

/** Access grant view from sharing endpoints */
export interface AccessGrantView {
  id: string;
  vaultId: string | null;
  folderId: string | null;
  itemId: string | null;
  userId: string | null;
  groupId: string | null;
  granteeName: string;
  granteeType: "user" | "group";
  accessCode: string;
  accessName: string;
  resourceAccessId: string;
  grantedBy: string | null;
  createdAt: string;
}
