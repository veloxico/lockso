-- 010: Performance indexes and schema fixes
-- Addresses audit findings: missing indexes, timestamp precision, FK defaults

-- ─── Performance indexes ────────────────────────────────────────────────────

-- Activity logs: composite index for resource-based queries
CREATE INDEX IF NOT EXISTS idx_activity_logs_resource
    ON activity_logs(resource_type, resource_id)
    WHERE resource_id IS NOT NULL;

-- Sessions: index for eviction query (ORDER BY last_activity_at)
CREATE INDEX IF NOT EXISTS idx_sessions_user_activity
    ON sessions(user_id, last_activity_at DESC);

-- Sessions: index for expired session cleanup
CREATE INDEX IF NOT EXISTS idx_sessions_refresh_expired
    ON sessions(refresh_token_expired_at);

-- Vaults: composite index for duplicate-name checks
CREATE INDEX IF NOT EXISTS idx_vaults_creator_name
    ON vaults(creator_id, name);

-- ─── Timestamp precision consistency ────────────────────────────────────────

-- Standardize attachments and activity_logs to TIMESTAMPTZ(3)
ALTER TABLE attachments
    ALTER COLUMN created_at TYPE TIMESTAMPTZ(3);

ALTER TABLE activity_logs
    ALTER COLUMN created_at TYPE TIMESTAMPTZ(3);

-- ─── Default UUID generators for consistency ────────────────────────────────

ALTER TABLE attachments
    ALTER COLUMN id SET DEFAULT gen_random_uuid();

ALTER TABLE activity_logs
    ALTER COLUMN id SET DEFAULT gen_random_uuid();
