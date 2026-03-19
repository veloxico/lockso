-- Webhook notification channels
CREATE TABLE webhooks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(100) NOT NULL,
    -- provider: "telegram", "slack", "discord", "custom"
    provider    VARCHAR(20) NOT NULL DEFAULT 'custom',
    -- encrypted URL / bot token depending on provider
    url_enc     TEXT NOT NULL,
    -- which events trigger this webhook (JSONB array of action codes)
    events      JSONB NOT NULL DEFAULT '["user.login", "user.login_failed", "item.created", "item.trashed"]',
    is_enabled  BOOLEAN NOT NULL DEFAULT TRUE,
    creator_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ(3) NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_webhooks_enabled ON webhooks(is_enabled) WHERE is_enabled = TRUE;
