---
name: rust-backend
description: Project-specific Rust backend conventions for the GOJEK-rust-api Axum service. Use when implementing or reviewing API, async, database, cache, and service code.
---

# Rust Backend Skill

Apply with [`CLAUDE.md`](../../../CLAUDE.md), [`AGENTS.md`](../../../AGENTS.md), and [`PRD.md`](../../../PRD.md). This project targets Rust + Axum/Tokio, SQLx/PostgreSQL/PostGIS, Redis, Serde, and Tracing.

## Implementation rules

- Keep the first working change minimal. Use standard library/platform features before adding crates. Add a dependency only for a clear current requirement.
- Organize by domain as the project grows; do not create empty layers/modules in advance. Keep HTTP handlers thin and business invariants testable without an HTTP server.
- Borrow inputs when ownership is not needed. Avoid unnecessary clones and broad `pub` surfaces.
- Use exhaustive enums for domain states. Validate trip transitions centrally; invalid state changes return explicit domain errors.
- Return `Result`; propagate failures with `?`. No `unwrap()` or `expect()` on production request paths. Never silently drop database/cache failures.
- Validate JSON and path/query input before domain work. Bound request sizes, pagination, string lengths, coordinate ranges, and retry-sensitive actions.
- Return a stable error envelope and appropriate HTTP status. Log internal diagnostics with request correlation, but never expose implementation details to clients.

## Async and state

- Run async work on Tokio; avoid blocking calls in request handlers.
- Share only required pools/clients through Axum state. Keep shared mutable state minimal.
- Apply dependency timeouts where operations can hang. Do not spawn detached tasks for work whose result is required by the request.

## PostgreSQL, SQLx, PostGIS

- PostgreSQL is durable source of truth for users, trips, fare snapshots, and payment records.
- Use SQLx bind parameters, migrations, transactions, constraints, and targeted indexes. Never concatenate user input into SQL.
- Use `timestamptz` for instants; money uses exact numeric or integer minor units, never floating point.
- Make booking/driver assignment atomic in PostgreSQL. Redis must never be the only guard against double assignment.
- Preserve fare-rule version and quote snapshot used by each trip; later configuration changes must not rewrite old trips.
- Use PostGIS only for actual spatial requirements; validate coordinates and coordinate reference assumptions explicitly.

## Redis

- Redis contains ephemeral driver locations, cache, and short-lived coordination only. Define TTLs and tolerate eviction/restart.
- Include observation time/freshness for driver locations. Stale locations must not participate in matching.
- Never store authoritative trip lifecycle or payment ledger only in Redis.

## Security and privacy

- Treat OTP, phone numbers, session tokens, identity documents, payment references, and precise locations as sensitive.
- Never hardcode secrets. Inject configuration through the environment; keep local secret files out of version control.
- Enforce authentication and per-resource authorization at the server boundary. Do not trust role, identity, price, or state from request payloads.
- Never log request bodies, tokens, OTPs, secrets, or precise coordinates. Use generic auth/OTP errors where enumeration is possible.
- For new authentication, upload, payment, or public endpoints, apply `../security/SKILL.md`.

## Testing and verification

- Put unit tests near pure domain logic; use integration tests for PostgreSQL/Redis behavior when available.
- Test success, invalid input, authorization, failure propagation, retry/idempotency, and concurrency where relevant.
- Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` when available. Report unavailable toolchain/dependencies honestly.
- Keep fixtures synthetic; never use real phone numbers, tokens, user data, or locations.
