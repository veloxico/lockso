export interface ActivityLogEntry {
  id: string;
  userId: string | null;
  userName: string | null;
  action: string;
  resourceType: string | null;
  resourceId: string | null;
  vaultId: string | null;
  clientIp: string | null;
  details: Record<string, unknown>;
  createdAt: string;
}

export interface PaginatedActivityLogs {
  data: ActivityLogEntry[];
  total: number;
  page: number;
  perPage: number;
}

export interface ActivityLogQuery {
  page?: number;
  perPage?: number;
  userId?: string;
  action?: string;
}
