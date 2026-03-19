-- API keys for programmatic access (CI/CD, scripts, automation)
CREATE TABLE api_keys (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          VARCHAR(100) NOT NULL,
    -- SHA-256 hash of the raw key (raw key shown only once at creation)
    key_hash      VARCHAR(64) NOT NULL UNIQUE,
    -- first 8 chars of the raw key for identification
    key_prefix    VARCHAR(8) NOT NULL,
    -- owner user
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- scoped permissions: "read" or "read_write"
    permission    VARCHAR(20) NOT NULL DEFAULT 'read',
    -- optional vault restriction (NULL = all accessible vaults)
    vault_id      UUID REFERENCES vaults(id) ON DELETE CASCADE,
    expires_at    TIMESTAMPTZ(3),
    last_used_at  TIMESTAMPTZ(3),
    is_enabled    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ(3) NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_user ON api_keys(user_id);
