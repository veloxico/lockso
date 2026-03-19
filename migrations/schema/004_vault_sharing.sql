-- Lockso 004: Vault sharing — user-level access control
-- Links users to vaults with a specific resource_access level.

-- ─── Vault User Accesses ───
CREATE TABLE vault_user_accesses (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id            UUID            NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    user_id             UUID            NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    resource_access_id  UUID            NOT NULL REFERENCES resource_accesses(id),
    granted_by          UUID            NOT NULL REFERENCES users(id),
    created_at          TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    UNIQUE(vault_id, user_id)
);

CREATE INDEX idx_vault_user_accesses_vault ON vault_user_accesses(vault_id);
CREATE INDEX idx_vault_user_accesses_user ON vault_user_accesses(user_id);

-- Update version
UPDATE _lockso_version SET db_schema_version = 4, updated_at = NOW()
WHERE app_version = '0.1.0';
