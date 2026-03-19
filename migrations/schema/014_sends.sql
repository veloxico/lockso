-- Phase 13: Secure Send — one-time encrypted sharing links.

CREATE TABLE sends (
    id              UUID PRIMARY KEY,
    creator_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_id       VARCHAR(44) NOT NULL UNIQUE,
    ciphertext_b64  TEXT NOT NULL,
    passphrase_hash VARCHAR(256),
    max_views       SMALLINT NOT NULL DEFAULT 1,
    view_count      SMALLINT NOT NULL DEFAULT 0,
    expires_at      TIMESTAMPTZ(3) NOT NULL,
    deleted_at      TIMESTAMPTZ(3),
    created_at      TIMESTAMPTZ(3) NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ(3) NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sends_creator ON sends(creator_id, created_at DESC);
CREATE INDEX idx_sends_access_id ON sends(access_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_sends_expires ON sends(expires_at) WHERE deleted_at IS NULL;
