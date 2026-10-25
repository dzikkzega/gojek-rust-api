# Agent Roles — GOJEK-rust-api

Use the smallest relevant role. Roles do not override [`CLAUDE.md`](CLAUDE.md), the PRD, permissions, or user scope.

## Backend Implementer
- Own Rust/Axum handlers, domain logic, SQLx access, and Redis integration for the assigned US/AC.
- Keep handlers thin; put validation and domain transitions in the smallest appropriate module.
- Add focused tests; preserve PostgreSQL as durable truth and Redis as ephemeral state.

## API Reviewer
- Check endpoint semantics, request validation, status codes, error envelope, idempotency, pagination, and compatibility against [`PRD.md`](PRD.md).
- Flag undocumented behavior or scope expansion; do not silently redesign the API.

## Database Reviewer
- Review SQL migrations, PostGIS use, constraints, indexes, transaction boundaries, and query parameterization.
- Verify concurrency-sensitive writes are atomic. Keep money exact; never use floating point.

## Security Reviewer
- Review authentication, authorization, OTP/session handling, rate limits, input boundaries, sensitive logging, document handling, and location privacy.
- Treat missing authorization, secret exposure, injection, and double assignment as blockers.

## Test Reviewer
- Ensure tests cover acceptance criteria and failure boundaries, not implementation details only.
- For booking/state changes, include retries/idempotency, invalid transitions, unauthorized access, and concurrent requests where applicable.

## Working protocol
1. Read the assigned User Story and AC, plus relevant PRD sections.
2. Inspect current code and conventions before editing.
3. Make the minimum implementation needed for the AC.
4. Run formatting, lint, and tests when the Rust toolchain is available.
5. Report changed files, checks run, blockers, and unresolved product decisions. Never claim an unrun check passed.
