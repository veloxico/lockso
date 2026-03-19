/** Vault member from GET /v1/sharing/:vaultId/members */
export interface VaultMember {
  id: string;
  userId: string;
  login: string;
  fullName: string;
  email: string | null;
  accessCode: string;
  accessName: string;
  resourceAccessId: string;
  grantedBy: string;
  createdAt: string;
}

/** Resource access level from GET /v1/sharing/access-levels */
export interface ResourceAccessLevel {
  id: string;
  name: string;
  code: string;
  permissions: string[];
  priority: number;
  isAccessOverrideAllowed: boolean;
  createdAt: string;
  updatedAt: string;
}
