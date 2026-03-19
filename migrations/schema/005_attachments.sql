-- Phase 7: File attachments
--
-- Each item can have multiple file attachments stored in S3.
-- Files are encrypted with AES-256-GCM before upload.
-- Original filename is encrypted; S3 key is an opaque UUID path.

CREATE TABLE IF NOT EXISTS attachments (
    id            UUID        PRIMARY KEY,
    item_id       UUID        NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    vault_id      UUID        NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    uploader_id   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Encrypted original filename (AES-256-GCM, stored as base64)
    name_enc      TEXT        NOT NULL,
    -- S3 object key (opaque, not encrypted — just a UUID-based path)
    storage_key   TEXT        NOT NULL UNIQUE,
    -- File metadata
    size_bytes    BIGINT      NOT NULL CHECK (size_bytes > 0),
    mime_type     VARCHAR(255) NOT NULL DEFAULT 'application/octet-stream',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_attachments_item_id ON attachments(item_id);
CREATE INDEX IF NOT EXISTS idx_attachments_vault_id ON attachments(vault_id);
