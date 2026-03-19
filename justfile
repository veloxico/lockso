# Lockso — Development Commands
# All builds run inside Docker. No local toolchains required.

# Default: show available commands
default:
    @just --list

# ─── Development ───

# Start all services in development mode (hot-reload)
dev:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build

# Start only infrastructure (PostgreSQL, Redis, MinIO)
infra:
    docker compose up postgres redis minio -d

# Stop all services
stop:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml down

# Stop and remove all volumes (DESTRUCTIVE)
nuke:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml down -v

# ─── Production Build ───

# Build production images
build:
    docker compose build

# Build only BE image
build-be:
    docker compose build lockso-be

# Build only FE image
build-fe:
    docker compose build lockso-fe

# Start in production mode
up:
    docker compose up -d

# ─── Backend ───

# Run cargo check inside Docker
check:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml run --rm lockso-be cargo check

# Run cargo clippy inside Docker
clippy:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml run --rm lockso-be cargo clippy -- -D warnings

# Run backend tests inside Docker
test-be:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml run --rm lockso-be cargo test

# Run cargo fmt check inside Docker
fmt-check:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml run --rm lockso-be cargo fmt --all -- --check

# ─── Frontend ───

# Run frontend lint inside Docker
lint-fe:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml run --rm lockso-fe npm run lint

# Run frontend type check inside Docker
typecheck:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml run --rm lockso-fe npm run typecheck

# Run frontend tests inside Docker
test-fe:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml run --rm lockso-fe npm run test

# ─── Database ───

# Run pending migrations
migrate:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml run --rm lockso-be cargo run --bin lockso -- migrate

# Show migration status
migrate-status:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml run --rm lockso-be cargo run --bin lockso -- migrate --status

# Connect to PostgreSQL via psql
psql:
    docker compose exec postgres psql -U lockso -d lockso

# Connect to Redis CLI
redis-cli:
    docker compose exec redis redis-cli

# ─── CI ───

# Run all checks (used in CI)
ci: fmt-check clippy test-be lint-fe typecheck test-fe

# ─── Logs ───

# Tail all logs
logs:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f

# Tail only BE logs
logs-be:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f lockso-be

# Tail only FE logs
logs-fe:
    docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f lockso-fe
