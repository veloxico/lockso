-- Phase 8: Activity log / Audit trail
--
-- Append-only log table for security-relevant events.
-- No sensitive data stored — only action, actor, target resource, and context.

CREATE TABLE IF NOT EXISTS activity_logs (
    id              UUID            PRIMARY KEY,
    user_id         UUID            REFERENCES users(id) ON DELETE SET NULL,
    action          VARCHAR(64)     NOT NULL,
    -- Target resource
    resource_type   VARCHAR(32),
    resource_id     UUID,
    vault_id        UUID            REFERENCES vaults(id) ON DELETE SET NULL,
    -- Context
    client_ip       VARCHAR(45),
    user_agent      TEXT,
    -- Optional structured metadata
    details         JSONB           NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activity_logs_created_at ON activity_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_activity_logs_user_id ON activity_logs(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_activity_logs_vault_id ON activity_logs(vault_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_activity_logs_action ON activity_logs(action);
