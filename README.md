# GOJEK Rust API

Minimal backend scaffold for the ride-hailing API. Product scope and MVP tasks: [`PRD.md`](PRD.md), [`TASK/TASKS.md`](TASK/TASKS.md).

## Stack

Axum + Tokio · PostgreSQL/PostGIS via SQLx · Redis · Serde · Tracing.

## Run

Install Rust stable (minimum 1.94 for the selected SQLx version), then run `cargo run`.

The API binds to `127.0.0.1:3000` by default. Set `BIND_ADDR` to override. `DATABASE_URL` and `REDIS_URL` are optional for local startup; when configured, `/api/v1/ready` checks both dependencies and returns `503` if either is unavailable or not configured. `/api/v1/health` is a liveness check and does not depend on them.

See [`.env.example`](.env.example) for local variable names. The application does not load `.env` files; export variables in the shell or development environment. Never put real credentials in `.env.example`.

## Endpoints

- `GET /api/v1/health` — process liveness.
- `GET /api/v1/ready` — readiness check for configured PostgreSQL and Redis dependencies.

## Checks

Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.

Domain endpoints, database schema/migrations, Docker setup, and provider integrations remain out of this minimal scaffold until their US/AC is implemented.
