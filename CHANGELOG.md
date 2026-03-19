# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.1] — 2026-03-21

### Added

- **Core password management** — create, read, update, delete credentials with encrypted field storage (AES-256-GCM)
- **Vault system** — Organization, Personal, and Private Shared vault types with full CRUD
- **Vault sharing** — Share vaults between users with granular permission control
- **User groups** — Team-based access management for vault sharing
- **User authentication** — Registration, login, session management with Argon2id password hashing
- **WebAuthn / FIDO2** — Two-factor authentication with hardware security keys and passkeys
- **Brute-force protection** — Redis-backed login rate limiting with configurable thresholds (enable/disable toggle)
- **Activity audit logs** — Full audit trail with user, action, resource filtering and pagination
- **File attachments** — Upload and manage files attached to vault items via S3-compatible storage
- **Trash & recovery** — Soft-delete items with restore and permanent delete support
- **Secure Send** — Time-limited, view-limited secret sharing links
- **Webhook integrations** — HTTP webhooks triggered on system events
- **API keys** — Token-based programmatic access for automation and integrations
- **Email notifications** — Support for SMTP, SendGrid, Amazon SES, Resend, Mailgun, Postmark, and Mandrill
- **Password expiration policies** — Configurable rotation schedules per vault
- **Import / Export** — Data migration support for moving between instances
- **Admin settings panel** — General settings, security, email configuration, and system management
- **Internationalization** — English and Russian language support (i18next)
- **Dark theme UI** — Modern interface built with React 19, Tailwind CSS 4, and shadcn/ui components
- **Timezone support** — Searchable timezone selector with UTC offset display
- **Docker deployment** — Development (hot-reload) and production (multi-arch distroless) Docker configurations
- **Auto-migrations** — Database schema applied automatically on startup (16 migration files)
- **OpenAPI documentation** — Swagger UI available in development mode

[0.0.1]: https://github.com/veloxico/lockso/releases/tag/v0.0.1
