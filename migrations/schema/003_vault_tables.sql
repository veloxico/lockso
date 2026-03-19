-- Lockso 003: Vault, folder, item, snapshot, and related tables
-- Core password management data layer.

-- ─── Vaults ───
CREATE TABLE vaults (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255)    NOT NULL,
    description     TEXT            NOT NULL DEFAULT '',
    vault_type_id   UUID            NOT NULL REFERENCES vault_types(id),
    creator_id      UUID            NOT NULL REFERENCES users(id),
    salt            VARCHAR(64)     NOT NULL,
    color_code      SMALLINT        NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vaults_creator ON vaults(creator_id);
CREATE INDEX idx_vaults_type ON vaults(vault_type_id);

-- ─── Folders ───
CREATE TABLE folders (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    name                VARCHAR(255)    NOT NULL,
    vault_id            UUID            NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    parent_folder_id    UUID            REFERENCES folders(id) ON DELETE CASCADE,
    ancestor_ids        JSONB           NOT NULL DEFAULT '[]'::jsonb,
    position            INTEGER         NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_folders_vault ON folders(vault_id);
CREATE INDEX idx_folders_parent ON folders(parent_folder_id);

-- ─── Items (Passwords) ───
-- Fields ending in _enc are AES-256-GCM encrypted (server-side).
-- Stored as base64-encoded [nonce][ciphertext+tag].
CREATE TABLE items (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id            UUID            NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    folder_id           UUID            REFERENCES folders(id) ON DELETE SET NULL,
    creator_id          UUID            NOT NULL REFERENCES users(id),
    name_enc            TEXT            NOT NULL,
    login_enc           TEXT            NOT NULL DEFAULT '',
    password_enc        TEXT            NOT NULL DEFAULT '',
    url_enc             TEXT            NOT NULL DEFAULT '',
    description_enc     TEXT            NOT NULL DEFAULT '',
    customs_enc         TEXT            NOT NULL DEFAULT '',
    tags                JSONB           NOT NULL DEFAULT '[]'::jsonb,
    search_hashes       JSONB           NOT NULL DEFAULT '{}'::jsonb,
    color_code          SMALLINT        NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_items_vault ON items(vault_id);
CREATE INDEX idx_items_folder ON items(folder_id);
CREATE INDEX idx_items_creator ON items(creator_id);
CREATE INDEX idx_items_search ON items USING GIN (search_hashes jsonb_path_ops);

-- ─── Snapshots (Password History) ───
-- Auto-created on every item create/update.
CREATE TABLE snapshots (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id         UUID            NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    vault_id        UUID            NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    name_enc        TEXT            NOT NULL,
    login_enc       TEXT            NOT NULL DEFAULT '',
    password_enc    TEXT            NOT NULL DEFAULT '',
    url_enc         TEXT            NOT NULL DEFAULT '',
    description_enc TEXT            NOT NULL DEFAULT '',
    customs_enc     TEXT            NOT NULL DEFAULT '',
    tags            JSONB           NOT NULL DEFAULT '[]'::jsonb,
    created_by      UUID            NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_snapshots_item ON snapshots(item_id);
CREATE INDEX idx_snapshots_vault ON snapshots(vault_id);

-- ─── Favorites (per-user) ───
CREATE TABLE favorites (
    id          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID            NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    item_id     UUID            NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    sort_order  INTEGER         NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, item_id)
);

CREATE INDEX idx_favorites_user ON favorites(user_id);

-- ─── Recent Views (per-user) ───
CREATE TABLE recent_views (
    id          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID            NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    item_id     UUID            NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    viewed_at   TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, item_id)
);

CREATE INDEX idx_recent_views_user ON recent_views(user_id);
CREATE INDEX idx_recent_views_viewed ON recent_views(viewed_at DESC);

-- ─── Colors (per-user per-entity) ───
CREATE TABLE colors (
    id          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID            NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    item_id     UUID            REFERENCES items(id) ON DELETE CASCADE,
    vault_id    UUID            REFERENCES vaults(id) ON DELETE CASCADE,
    code        SMALLINT        NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_colors_target CHECK (
        (item_id IS NOT NULL AND vault_id IS NULL) OR
        (item_id IS NULL AND vault_id IS NOT NULL)
    )
);

CREATE INDEX idx_colors_user ON colors(user_id);
CREATE INDEX idx_colors_item ON colors(user_id, item_id) WHERE item_id IS NOT NULL;
CREATE INDEX idx_colors_vault ON colors(user_id, vault_id) WHERE vault_id IS NOT NULL;

-- Update version
UPDATE _lockso_version SET db_schema_version = 3, updated_at = NOW()
WHERE app_version = '0.1.0';
