/** Vault list item from GET /v1/vaults */
export interface VaultListItem {
  id: string;
  name: string;
  description: string;
  vaultTypeId: string;
  colorCode: number;
  itemCount: number;
  folderCount: number;
  createdAt: string;
}

/** Vault detail from GET /v1/vaults/:id */
export interface VaultView {
  id: string;
  name: string;
  description: string;
  vaultTypeId: string;
  creatorId: string;
  colorCode: number;
  itemCount: number;
  folderCount: number;
  createdAt: string;
  updatedAt: string;
}

/** POST /v1/vaults */
export interface CreateVaultRequest {
  name: string;
  description?: string;
  vaultTypeId: string;
  colorCode?: number;
}

/** PUT /v1/vaults/:id */
export interface UpdateVaultRequest {
  name?: string;
  description?: string;
  colorCode?: number;
}

/** Folder view */
export interface FolderView {
  id: string;
  name: string;
  vaultId: string;
  parentFolderId: string | null;
  position: number;
  itemCount: number;
  createdAt: string;
  updatedAt: string;
}

/** Folder tree node */
export interface FolderTreeNode {
  id: string;
  name: string;
  parentFolderId: string | null;
  position: number;
  itemCount: number;
  children: FolderTreeNode[];
}

/** POST /v1/folders */
export interface CreateFolderRequest {
  name: string;
  vaultId: string;
  parentFolderId?: string;
  position?: number;
}

/** Item list entry (no password) */
export interface ItemListEntry {
  id: string;
  vaultId: string;
  folderId: string | null;
  name: string;
  login: string;
  url: string;
  tags: string[];
  colorCode: number;
  isFavorite: boolean;
  createdAt: string;
  updatedAt: string;
}

/** Item detail (full, with password) */
export interface ItemView {
  id: string;
  vaultId: string;
  folderId: string | null;
  creatorId: string;
  name: string;
  login: string;
  password: string;
  url: string;
  description: string;
  customs: CustomField[];
  tags: string[];
  colorCode: number;
  isFavorite: boolean;
  passwordChangedAt: string;
  createdAt: string;
  updatedAt: string;
}

/** Custom field */
export interface CustomField {
  name: string;
  value: string;
  type: "text" | "password" | "url" | "email" | "totp";
}

/** POST /v1/items */
export interface CreateItemRequest {
  vaultId: string;
  folderId?: string;
  name: string;
  login?: string;
  password?: string;
  url?: string;
  description?: string;
  customs?: CustomField[];
  tags?: string[];
  colorCode?: number;
}

/** PUT /v1/items/:id */
export interface UpdateItemRequest {
  name?: string;
  login?: string;
  password?: string;
  url?: string;
  description?: string;
  customs?: CustomField[];
  tags?: string[];
  colorCode?: number;
  folderId?: string;
}

/** Snapshot list entry */
export interface SnapshotListEntry {
  id: string;
  itemId: string;
  name: string;
  login: string;
  createdBy: string;
  createdAt: string;
}

/** Snapshot detail */
export interface SnapshotView {
  id: string;
  itemId: string;
  name: string;
  login: string;
  password: string;
  url: string;
  description: string;
  customs: CustomField[];
  tags: string[];
  createdBy: string;
  createdAt: string;
}

/** Search request */
export interface SearchRequest {
  query: string;
  vaultId?: string;
}

/** Favorite toggle response */
export interface FavoriteResponse {
  isFavorite: boolean;
}

/** Trash list entry */
export interface TrashListEntry {
  id: string;
  vaultId: string;
  folderId: string | null;
  name: string;
  login: string;
  url: string;
  colorCode: number;
  vaultName: string;
  deletedAt: string;
  createdAt: string;
}

/** Trash count response */
export interface TrashCountResponse {
  count: number;
}

/** Health report */
export interface HealthReport {
  totalItems: number;
  weakCount: number;
  reusedCount: number;
  oldCount: number;
  breachedCount: number;
  score: number;
  items: HealthItem[];
  reuseGroups: ReuseGroup[];
}

export interface HealthItem {
  id: string;
  vaultId: string;
  name: string;
  login: string;
  url: string;
  vaultName: string;
  colorCode: number;
  strength: number;
  isWeak: boolean;
  isReused: boolean;
  isOld: boolean;
  isBreached: boolean;
  breachCount: number;
  passwordAgeDays: number;
  passwordChangedAt: string;
  passwordHashPrefix: string;
}

export interface ReuseGroup {
  itemIds: string[];
  count: number;
}

/** Secure Send list entry */
export interface SendListEntry {
  id: string;
  accessId: string;
  hasPassphrase: boolean;
  maxViews: number;
  viewCount: number;
  expiresAt: string;
  isExpired: boolean;
  isConsumed: boolean;
  createdAt: string;
}

export interface CreateSendRequest {
  ciphertextB64: string;
  passphrase?: string;
  maxViews?: number;
  ttlHours?: number;
}

export interface CreateSendResponse {
  id: string;
  accessId: string;
}

export interface SendPublicMeta {
  hasPassphrase: boolean;
}

export interface SendAccessView {
  ciphertextB64: string;
}

/** File attachment */
export interface AttachmentView {
  id: string;
  itemId: string;
  name: string;
  sizeBytes: number;
  mimeType: string;
  uploaderId: string;
  createdAt: string;
}
