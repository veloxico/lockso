-- Lockso 008: Email notification settings
-- Stores encrypted email provider configuration.

CREATE TABLE email_settings (
    id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    provider        VARCHAR(32)     NOT NULL,  -- smtp, sendgrid, ses, resend, mailgun, postmark, mandrill
    is_enabled      BOOLEAN         NOT NULL DEFAULT FALSE,
    from_name       VARCHAR(255)    NOT NULL DEFAULT 'Lockso',
    from_email      VARCHAR(255)    NOT NULL DEFAULT '',
    -- Provider config stored as encrypted JSON (contains API keys, host, etc.)
    config_enc      TEXT            NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ(3)  NOT NULL DEFAULT NOW()
);

-- Only one email config row allowed
CREATE UNIQUE INDEX idx_email_settings_singleton ON email_settings((TRUE));

-- Update version
UPDATE _lockso_version SET db_schema_version = 8, updated_at = NOW()
WHERE app_version = '0.1.0';
