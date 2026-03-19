export interface SessionView {
  id: string;
  authMethod: string;
  clientType: string;
  clientIp: string | null;
  userAgent: string | null;
  accessTokenExpiredAt: string;
  lastActivityAt: string;
  isCurrent: boolean;
  createdAt: string;
}
