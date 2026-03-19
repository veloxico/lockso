-- Lockso 007: Password expiration tracking
-- Adds password_changed_at to items for password age monitoring.

ALTER TABLE items
    ADD COLUMN password_changed_at TIMESTAMPTZ(3) NOT NULL DEFAULT NOW();

-- Backfill existing items: use created_at as initial password_changed_at
UPDATE items SET password_changed_at = created_at;

-- Update version
UPDATE _lockso_version SET db_schema_version = 7, updated_at = NOW()
WHERE app_version = '0.1.0';
