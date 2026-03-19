/** Admin user list item from GET /v1/users */
export interface AdminUserListItem {
  id: string;
  login: string;
  email: string | null;
  fullName: string;
  signupType: string;
  roleId: string;
  roleName: string;
  roleCode: string;
  isBlocked: boolean;
  lastLoginAt: string | null;
  createdAt: string;
}

/** User role from GET /v1/users/roles */
export interface UserRoleView {
  id: string;
  name: string;
  code: string;
  permissions: string[];
  authSettings: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
}

/** Settings from GET /v1/settings */
export interface AppSettings {
  id: string;
  session: SessionSettings;
  email: Record<string, unknown>;
  sso: Record<string, unknown>;
  search: Record<string, unknown>;
  favicon: Record<string, unknown>;
  interface: InterfaceSettings;
  notification: Record<string, unknown>;
  customBanner: Record<string, unknown>;
  userLockout: UserLockoutSettings;
  activityLog: Record<string, unknown>;
  browserExtension: Record<string, unknown>;
  authPasswordComplexity: PasswordComplexity;
  masterPasswordComplexity: PasswordComplexity;
  vault: Record<string, unknown>;
  task: Record<string, unknown>;
  user: Record<string, unknown>;
  internal: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
}

export interface SessionSettings {
  accessTokenTtl: number;
  refreshTokenTtl: number;
  inactivityTtl: number;
  csrfTokenTtl: number;
}

export interface PasswordComplexity {
  minLength: number;
  requireUppercase: boolean;
  requireLowercase: boolean;
  requireDigits: boolean;
  requireSpecial: boolean;
}

export interface UserLockoutSettings {
  enabled: boolean;
  maxAttempts: number;
  windowSeconds: number;
  lockoutSeconds: number;
}

export interface InterfaceSettings {
  defaultLanguage: string;
  defaultTimezone: string;
}
