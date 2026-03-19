/** Health check response from GET /v1/app/health-check */
export interface HealthResponse {
  status: "healthy" | "degraded";
  version: string;
  isBootstrapped: boolean;
  database: ServiceStatus;
  redis: ServiceStatus;
  storage: ServiceStatus;
}

export interface ServiceStatus {
  status: "ok" | "error";
  error?: string;
}

/** POST /v1/users/login request */
export interface LoginRequest {
  login: string;
  password: string;
  clientType?: string;
}

/** POST /v1/users/login response */
export interface LoginResponse {
  accessToken: string;
  refreshToken: string;
  accessTokenExpiredAt: string;
  refreshTokenExpiredAt: string;
  user: UserView;
  isTwoFactorAuthRequired: boolean;
  isMasterKeyRequired: boolean;
}

/** POST /v1/users/register request */
export interface RegisterRequest {
  login: string;
  password: string;
  email?: string;
  fullName?: string;
  masterKeyHash?: string;
  keysPublic?: string;
  keysPrivateEncrypted?: string;
}

/** User view (safe fields only) */
export interface UserView {
  id: string;
  login: string;
  email: string | null;
  fullName: string;
  signupType: string;
  roleId: string;
  isBlocked: boolean;
  lastLoginAt: string | null;
  createdAt: string;
  updatedAt: string;
}

/** Standard API error response */
export interface ApiErrorResponse {
  code: string;
  message: string;
}
