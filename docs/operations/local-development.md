# Local development and database operations

This is a local developer runbook, not a production deployment or backup policy.

## Prerequisites

- Docker Desktop engine running.
- Rust toolchain selected by `rust-toolchain.toml` (stable + rustfmt + clippy); current `Cargo.toml` requires Rust 1.94 or newer.
- Copy `.env.example` to ignored `.env`; values are local development only.
- Compose uses `postgis/postgis:17-3.6`, binds PostgreSQL to `127.0.0.1:5433` by default, and stores data in named volume `gojek-rust-api_postgres_data`.

## Start and inspect PostgreSQL/PostGIS

From repository root:

```bash
docker compose config
docker compose up -d postgres
docker compose ps
docker compose exec -T postgres pg_isready -U gojek -d gojek_api
docker compose exec -T postgres psql -U gojek -d gojek_api -c "SELECT PostGIS_Version();"
```

Wait until Compose reports `healthy`. If port 5433 is occupied, change `POSTGRES_PORT` in `.env` and keep the `DATABASE_URL` host port synchronized. Do not stop another project's container to free a port without its owner's approval.

## SQLx migrations

Migration files in `migrations/` run in timestamp order. SQLx records applied versions in `_sqlx_migrations` and `sqlx migrate run` skips versions already applied.

```bash
sqlx migrate info
sqlx migrate run
sqlx migrate info
```

Inspect extension/tables:

```bash
docker compose exec -T postgres psql -U gojek -d gojek_api -c "SELECT extname, extversion FROM pg_extension WHERE extname = 'postgis';"
docker compose exec -T postgres psql -U gojek -d gojek_api -c "\\dt"
```

Do not use `docker compose down -v`, `DROP DATABASE` or `sqlx database reset` for routine cleanup: these erase local database data. `docker compose stop postgres` preserves the container/volume. `docker compose down` removes container/network but preserves the named volume.

## Run and smoke-test API

The application currently reads process environment; it does not parse `.env` itself. Export the variables via shell/IDE environment integration, then run:

```bash
cargo run
```

- `GET /api/v1/health` checks process liveness only.
- `GET /api/v1/ready` checks configured PostgreSQL and Redis; the current implementation requires both, so it returns `503` while Redis is not configured.
- Redis is not included in the current Compose setup.

## Change workflow

- Add a new forward migration for schema changes; never edit an already-applied migration in a shared database.
- Update the ERD, OpenAPI contract, and affected ACs with behavior changes.
- Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`; report if unavailable rather than claiming success.
- Review `git diff`, secrets, generated files and `.env` ignore status before commit.

## Production remains TBD

No production host, TLS termination, secret manager, backup schedule/retention, restore drill, monitoring target, scale policy or SLO is established here. Resolve these in the decision register before deployment.
