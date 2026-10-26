# GOJEK Rust API

Minimal backend scaffold for the ride-hailing API. Start with the [documentation index](docs/README.md), [product requirements](PRD.md), and [MVP task backlog](TASK/TASKS.md). API contract: [OpenAPI draft](docs/api/openapi.yaml). Database: [schema migrations](migrations/), [visual ERD](docs/database-erd.svg), and [relationship notes](docs/database-relationships.md). Operations, architecture and security: [local runbook](docs/operations/local-development.md), [architecture](docs/architecture.md), [testing/release gates](docs/testing-and-release.md), [implementation status](docs/implementation-status.md), [data handling](docs/security/data-handling.md), and [open decisions](docs/decisions/open-decisions.md). *OpenAPI and policy documents remain drafts until their TBDs are approved.*

## Stack

Axum + Tokio · PostgreSQL/PostGIS via SQLx · Redis · Serde · Tracing.

## Local PostgreSQL + PostGIS

Install Docker Desktop and start its engine. Copy `.env.example` to `.env`, then start PostgreSQL/PostGIS:

```bash
docker compose up -d postgres
docker compose ps
```

The database listens on `127.0.0.1:${POSTGRES_PORT:-5433}` and persists in the named `postgres_data` volume. The default credentials in `.env.example` are for local development only. Keep `.env` private; change the password before sharing the environment or exposing the port beyond localhost.

Apply the PostGIS extension and schema migrations:

```bash
cargo install sqlx-cli --no-default-features --features postgres,rustls
sqlx migrate run
```

This installs the PostGIS extension, then creates the MVP tables. Review the [ERD](docs/database-relationships.md) before applying schema changes.

Stop the container without deleting data with `docker compose stop postgres`. To remove the container, use `docker compose down`; the named volume remains. `docker compose down -v` permanently deletes the database volume—do not run it unless you intend to erase local database data.

## Run the API

Install Rust stable (minimum 1.94 for the selected SQLx version), export variables from `.env` in your shell/development environment, then run `cargo run`.

The API binds to `127.0.0.1:3000` by default. Set `BIND_ADDR` to override. `DATABASE_URL` and `REDIS_URL` are optional for local startup; `/api/v1/ready` checks configured dependencies and returns `503` when either dependency is missing or unavailable. `/api/v1/health` is a liveness check and does not depend on them. Redis is not part of the current Compose setup.

Never commit `.env` or real credentials. `.env.example` contains development-only placeholders.

Verify PostGIS is installed with:

```bash
docker compose exec postgres psql -U gojek -d gojek_api -c "SELECT PostGIS_Version();"
```

The command uses the default development database/user; update it if `.env` overrides them.

## Database schema

Review the full table relationships in [`docs/database-relationships.md`](docs/database-relationships.md). The SQLx migrations are in `migrations/`; apply them with `sqlx migrate run` after PostgreSQL is healthy.

Schema overview:

- `users` → auth sessions, driver applications, rider quotes/trips, ratings, and audit actors.
- Driver application → documents; approved application → driver profile → vehicles.
- Service area → versioned fare rules → immutable quote/trip fare snapshots.
- Rider quote → at most one trip; trip → status events, at most one payment, and participant ratings.
- Driver positions are ephemeral in Redis, not durable PostgreSQL rows.

The schema is MVP scope; OTP challenges, support, notifications, and non-cash provider payments are intentionally excluded for now.
## Endpoints

- `GET /api/v1/health` — process liveness.
- `GET /api/v1/ready` — readiness check for configured PostgreSQL and Redis dependencies.

## Checks

Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.

Domain endpoints and provider integrations remain out of this minimal scaffold until their US/AC is implemented.
