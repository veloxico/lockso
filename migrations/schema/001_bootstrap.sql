-- Lockso V001: Bootstrap migration
-- Creates internal tracking tables for version and migration management.

-- Version tracking
CREATE TABLE IF NOT EXISTS _lockso_version (
    app_version     TEXT        PRIMARY KEY,
    db_schema_version BIGINT   NOT NULL DEFAULT 1,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Data migration tracking (Rust-side DML migrations)
CREATE TABLE IF NOT EXISTS _lockso_data_migrations (
    id          SERIAL      PRIMARY KEY,
    name        TEXT        NOT NULL UNIQUE,
    status      TEXT        NOT NULL DEFAULT 'pending'
                            CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    started_at  TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Insert initial version
INSERT INTO _lockso_version (app_version, db_schema_version, updated_at)
VALUES ('0.1.0', 1, NOW())
ON CONFLICT (app_version) DO NOTHING;
