-- Lockso 009: WebAuthn/FIDO2 credential storage
-- Stores registered authenticator credentials for passwordless/2FA.

CREATE TABLE webauthn_credentials (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID            NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id   TEXT            NOT NULL UNIQUE,
    public_key      TEXT            NOT NULL,
    sign_count      BIGINT          NOT NULL DEFAULT 0,
    transports      JSONB           NOT NULL DEFAULT '[]'::jsonb,
    device_name     VARCHAR(255)    NOT NULL DEFAULT '',
    aaguid          VARCHAR(64)     NOT NULL DEFAULT '',
    backed_up       BOOLEAN         NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    last_used_at    TIMESTAMPTZ(3)
);

CREATE INDEX idx_webauthn_user ON webauthn_credentials(user_id);
CREATE INDEX idx_webauthn_cred_id ON webauthn_credentials(credential_id);

-- Update version
UPDATE _lockso_version SET db_schema_version = 9, updated_at = NOW()
WHERE app_version = '0.1.0';
