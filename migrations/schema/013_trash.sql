-- Phase 11: Soft-delete / Trash for items
-- Adds deleted_at / deleted_by to items for soft-delete functionality.

ALTER TABLE items
    ADD COLUMN deleted_at    TIMESTAMPTZ(3)  DEFAULT NULL,
    ADD COLUMN deleted_by    UUID            REFERENCES users(id) ON DELETE SET NULL;

-- Partial index: fast trash listing (only trashed items)
CREATE INDEX idx_items_deleted_at ON items(deleted_at) WHERE deleted_at IS NOT NULL;

-- Partial index: ensure all existing non-trash queries stay fast
CREATE INDEX idx_items_vault_not_deleted ON items(vault_id, created_at DESC) WHERE deleted_at IS NULL;

-- Add trash settings category
ALTER TABLE settings ADD COLUMN IF NOT EXISTS trash JSONB NOT NULL DEFAULT '{"retentionDays": 30, "autoEmptyEnabled": true}';

-- Update allowed settings categories (handled in Rust code, not SQL)
