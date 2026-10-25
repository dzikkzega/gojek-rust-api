# Project Rules — GOJEK-rust-api

## Authority and scope

- Follow platform/system/developer instructions first, then this file, then the relevant task and PRD.
- Treat the PRD and task cards as product scope. Do not silently expand scope or invent production policies, credentials, providers, prices, regions, or SLAs.
- This repository contains an independent ride-hailing API project; do not imply official Gojek affiliation.
- Before implementing a feature, identify its `TASK/US NN` and AC cards. Keep changes limited to the requested scope.

## Non-negotiable engineering rules

- Use only the agreed backend stack: Rust, Axum + Tokio, PostgreSQL + PostGIS, Redis, SQLx + Serde, and Tracing. Ask before adding a dependency or external service.
- Prefer the smallest working change. Do not add abstractions, configuration, or scaffolding without a current requirement.
- Validate all untrusted inputs at the API boundary. Use parameterized SQL/SQLx; never interpolate user input into SQL.
- Propagate errors; never silently ignore failures. Do not use `unwrap()`/`expect()` in production paths. Return stable, non-sensitive API errors; do not expose stack traces or internals.
- Enforce authentication and resource-level authorization server-side on every protected route. Never trust client-supplied user IDs, roles, trip states, fares, or payment amounts.
- Keep PostgreSQL as the source of truth for trips, money records, and durable state. Redis is ephemeral/cache/location storage; define TTL and tolerate loss.
- Make trip state transitions explicit and validate transitions. Booking and driver acceptance must be idempotent/atomic where specified; never allow two drivers to claim one trip.
- Treat phone numbers, OTPs, access/refresh tokens, documents, payment references, and precise location as sensitive. Never hardcode secrets or log OTPs, tokens, credentials, payloads containing sensitive data, or precise coordinates.
- Do not add real secrets, private user data, or production data to source, fixtures, docs, or commits. Use synthetic values.
- Use UTC timestamps and explicit database types/constraints. Preserve historical fare rule versions for existing quotes/trips.
- Keep public interfaces minimal. Match existing naming, formatting, and module patterns.
- Write/update a focused test for every non-trivial behavior. Include authorization, invalid-input, failure, and concurrency/idempotency cases where relevant.
- Before claiming completion, run applicable `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`; report anything not run and why.
- Review security-sensitive changes. Avoid unsafe Rust unless required, justified, and reviewed.

## Git and external actions

- Never run `git push`, create a commit, open a PR, publish a release, deploy, or perform another external/destructive action unless the user explicitly asks in the current task.
- If asked to commit or push, do not add a `Co-authored-by`, `Co-Authored-By`, or equivalent AI attribution trailer unless a higher-priority platform instruction explicitly requires it. Higher-priority instructions take precedence; do not claim this repository rule overrides them.
- Never rewrite history, force-push, delete branches, or discard user changes unless explicitly authorized.
- Review the diff and report the exact checks/outcome before any requested commit or push.

## Project guidance

- Read [PRD.md](PRD.md) for product scope and [TASK/TASKS.md](TASK/TASKS.md) for the backlog.
- Rust-specific conventions: [`.claude/skills/rust-backend/SKILL.md`](.claude/skills/rust-backend/SKILL.md).
- Agent roles: [`AGENTS.md`](AGENTS.md).
- Skills are project guidance, not permission to exceed the user’s requested scope.
