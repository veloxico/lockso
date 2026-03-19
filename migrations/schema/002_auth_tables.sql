-- Lockso 002: Auth tables
-- Creates core authentication tables: users, sessions, csrf_tokens,
-- settings, user_roles, resource_accesses.

-- ─── User Roles ───
CREATE TABLE user_roles (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255)    NOT NULL,
    code            VARCHAR(100)    NOT NULL UNIQUE,
    permissions     JSONB           NOT NULL DEFAULT '[]'::jsonb,
    auth_settings   JSONB           NOT NULL DEFAULT '{}'::jsonb,
    manageable_user_roles JSONB     NOT NULL DEFAULT '[]'::jsonb,
    offline_access  JSONB           NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_roles_code ON user_roles(code);

-- ─── Users ───
CREATE TABLE users (
    id                      UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    login                   VARCHAR(255)    NOT NULL UNIQUE,
    password_hash           VARCHAR(512)    NOT NULL,
    email                   VARCHAR(255),
    full_name               VARCHAR(255)    NOT NULL DEFAULT '',
    master_key_options      JSONB           NOT NULL DEFAULT '{}'::jsonb,
    master_key_hash         VARCHAR(512),
    keys_public             TEXT,
    keys_private_encrypted  TEXT,
    signup_type             VARCHAR(50)     NOT NULL DEFAULT 'Default',
    role_id                 UUID            NOT NULL REFERENCES user_roles(id),
    auth_settings           JSONB           NOT NULL DEFAULT '{}'::jsonb,
    blocked_ips             JSONB           NOT NULL DEFAULT '[]'::jsonb,
    interface_settings      JSONB           NOT NULL DEFAULT '{}'::jsonb,
    client_settings         JSONB           NOT NULL DEFAULT '{}'::jsonb,
    password_hash_history   JSONB           NOT NULL DEFAULT '[]'::jsonb,
    is_blocked              BOOLEAN         NOT NULL DEFAULT FALSE,
    last_login_at           TIMESTAMPTZ(3),
    created_at              TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_login ON users(login);
CREATE INDEX idx_users_email ON users(email) WHERE email IS NOT NULL;
CREATE INDEX idx_users_role_id ON users(role_id);

-- ─── Sessions ───
CREATE TABLE sessions (
    id                          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                     UUID            NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token_hash           VARCHAR(256)    NOT NULL,
    refresh_token_hash          VARCHAR(256)    NOT NULL,
    auth_method                 VARCHAR(50)     NOT NULL DEFAULT 'Local',
    is_two_factor_auth_required BOOLEAN         NOT NULL DEFAULT FALSE,
    client_type                 VARCHAR(50)     NOT NULL DEFAULT 'Web',
    client_ip                   VARCHAR(45),
    user_agent                  TEXT,
    access_token_expired_at     TIMESTAMPTZ(3)  NOT NULL,
    refresh_token_expired_at    TIMESTAMPTZ(3)  NOT NULL,
    last_activity_at            TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    pin_code_hash               VARCHAR(256),
    is_pin_code_required        BOOLEAN         NOT NULL DEFAULT FALSE,
    webauthn_challenge          VARCHAR(512),
    last_authentications        JSONB           NOT NULL DEFAULT '{}'::jsonb,
    attributes                  JSONB           NOT NULL DEFAULT '{}'::jsonb,
    created_at                  TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_access_token_hash ON sessions(access_token_hash);
CREATE INDEX idx_sessions_refresh_token_hash ON sessions(refresh_token_hash);
CREATE INDEX idx_sessions_expired ON sessions(access_token_expired_at);

-- ─── CSRF Tokens ───
CREATE TABLE csrf_tokens (
    id          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash  VARCHAR(256)    NOT NULL,
    session_id  UUID            NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    expired_at  TIMESTAMPTZ(3)  NOT NULL,
    created_at  TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_csrf_tokens_session_id ON csrf_tokens(session_id);
CREATE INDEX idx_csrf_tokens_hash ON csrf_tokens(token_hash);
CREATE INDEX idx_csrf_tokens_expired ON csrf_tokens(expired_at);

-- ─── Settings (singleton — one row) ───
CREATE TABLE settings (
    id                          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    session                     JSONB           NOT NULL DEFAULT '{}'::jsonb,
    email                       JSONB           NOT NULL DEFAULT '{}'::jsonb,
    sso                         JSONB           NOT NULL DEFAULT '{}'::jsonb,
    search                      JSONB           NOT NULL DEFAULT '{}'::jsonb,
    favicon                     JSONB           NOT NULL DEFAULT '{}'::jsonb,
    interface                   JSONB           NOT NULL DEFAULT '{}'::jsonb,
    notification                JSONB           NOT NULL DEFAULT '{}'::jsonb,
    custom_banner               JSONB           NOT NULL DEFAULT '{}'::jsonb,
    user_lockout                JSONB           NOT NULL DEFAULT '{}'::jsonb,
    activity_log                JSONB           NOT NULL DEFAULT '{}'::jsonb,
    browser_extension           JSONB           NOT NULL DEFAULT '{}'::jsonb,
    auth_password_complexity    JSONB           NOT NULL DEFAULT '{}'::jsonb,
    master_password_complexity  JSONB           NOT NULL DEFAULT '{}'::jsonb,
    vault                       JSONB           NOT NULL DEFAULT '{}'::jsonb,
    task                        JSONB           NOT NULL DEFAULT '{}'::jsonb,
    "user"                      JSONB           NOT NULL DEFAULT '{}'::jsonb,
    internal                    JSONB           NOT NULL DEFAULT '{}'::jsonb,
    created_at                  TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

-- ─── Resource Access Levels ───
CREATE TABLE resource_accesses (
    id                          UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    name                        VARCHAR(255)    NOT NULL,
    code                        VARCHAR(100)    NOT NULL UNIQUE,
    permissions                 JSONB           NOT NULL DEFAULT '[]'::jsonb,
    priority                    INTEGER         NOT NULL DEFAULT 0,
    is_access_override_allowed  BOOLEAN         NOT NULL DEFAULT FALSE,
    created_at                  TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_resource_accesses_code ON resource_accesses(code);

-- ─── Vault Types (needed for bootstrap) ───
CREATE TABLE vault_types (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255)    NOT NULL,
    code            VARCHAR(100)    NOT NULL UNIQUE,
    allowed_users   JSONB           NOT NULL DEFAULT '[]'::jsonb,
    allowed_groups  JSONB           NOT NULL DEFAULT '[]'::jsonb,
    allowed_roles   JSONB           NOT NULL DEFAULT '[]'::jsonb,
    created_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

-- Update version
UPDATE _lockso_version SET db_schema_version = 2, updated_at = NOW()
WHERE app_version = '0.1.0';
