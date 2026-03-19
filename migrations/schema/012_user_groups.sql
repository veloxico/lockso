-- =====================================================
-- 012: User Groups + Unified Access Grants
-- =====================================================

-- ─── User Groups ───
CREATE TABLE user_groups (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255)    NOT NULL,
    description     TEXT            NOT NULL DEFAULT '',
    creator_id      UUID            REFERENCES users(id) ON DELETE SET NULL,
    is_active       BOOLEAN         NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_user_groups_name ON user_groups(LOWER(name));
CREATE INDEX idx_user_groups_creator ON user_groups(creator_id);

-- ─── Group Membership ───
CREATE TABLE user_group_members (
    id          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id    UUID            NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    user_id     UUID            NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    added_by    UUID            REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    UNIQUE(group_id, user_id)
);

CREATE INDEX idx_ugm_group ON user_group_members(group_id);
CREATE INDEX idx_ugm_user ON user_group_members(user_id);

-- ─── Unified Resource Access Grants ───
-- Replaces vault_user_accesses with a polymorphic grants table
-- that supports vault/folder/item × user/group combinations.
CREATE TABLE resource_access_grants (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Target resource (exactly one must be set)
    vault_id            UUID            REFERENCES vaults(id) ON DELETE CASCADE,
    folder_id           UUID            REFERENCES folders(id) ON DELETE CASCADE,
    item_id             UUID            REFERENCES items(id) ON DELETE CASCADE,
    -- Grantee (exactly one must be set)
    user_id             UUID            REFERENCES users(id) ON DELETE CASCADE,
    group_id            UUID            REFERENCES user_groups(id) ON DELETE CASCADE,
    -- Access level
    resource_access_id  UUID            NOT NULL REFERENCES resource_accesses(id),
    granted_by          UUID            REFERENCES users(id) ON DELETE SET NULL,
    created_at          TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),

    -- Exactly one target resource
    CONSTRAINT chk_one_resource CHECK (
        (vault_id IS NOT NULL)::int +
        (folder_id IS NOT NULL)::int +
        (item_id IS NOT NULL)::int = 1
    ),
    -- Exactly one grantee
    CONSTRAINT chk_one_grantee CHECK (
        (user_id IS NOT NULL)::int +
        (group_id IS NOT NULL)::int = 1
    ),
    -- No duplicate grants for the same resource+grantee
    UNIQUE NULLS NOT DISTINCT (vault_id, folder_id, item_id, user_id, group_id)
);

-- Partial indexes for fast permission lookups
CREATE INDEX idx_rag_vault_user ON resource_access_grants(vault_id, user_id)
    WHERE vault_id IS NOT NULL AND user_id IS NOT NULL;
CREATE INDEX idx_rag_vault_group ON resource_access_grants(vault_id, group_id)
    WHERE vault_id IS NOT NULL AND group_id IS NOT NULL;
CREATE INDEX idx_rag_folder_user ON resource_access_grants(folder_id, user_id)
    WHERE folder_id IS NOT NULL AND user_id IS NOT NULL;
CREATE INDEX idx_rag_folder_group ON resource_access_grants(folder_id, group_id)
    WHERE folder_id IS NOT NULL AND group_id IS NOT NULL;
CREATE INDEX idx_rag_item_user ON resource_access_grants(item_id, user_id)
    WHERE item_id IS NOT NULL AND user_id IS NOT NULL;
CREATE INDEX idx_rag_item_group ON resource_access_grants(item_id, group_id)
    WHERE item_id IS NOT NULL AND group_id IS NOT NULL;
CREATE INDEX idx_rag_user ON resource_access_grants(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_rag_group ON resource_access_grants(group_id) WHERE group_id IS NOT NULL;

-- ─── Migrate existing vault_user_accesses → resource_access_grants ───
INSERT INTO resource_access_grants (id, vault_id, user_id, resource_access_id, granted_by, created_at, updated_at)
SELECT id, vault_id, user_id, resource_access_id, granted_by, created_at, updated_at
FROM vault_user_accesses;

-- Keep vault_user_accesses for now (drop in future migration after verification)
