# Testing and release gates

This plan maps PRD acceptance criteria to verification. It does not claim tests currently exist or pass; the Rust toolchain is not available in the current development environment at the time of writing.

## Test layers

| Layer | Focus | Minimum examples |
|---|---|---|
| Domain unit | Pure rule correctness | State transition accepted/rejected; fare arithmetic exactness; quote expiry; cancellation eligibility |
| Axum API | Public contract and authz | Status/error schema; input validation; missing/invalid token; owner vs non-owner resource access |
| PostgreSQL/PostGIS integration | Durable invariants | Migrations from empty DB; constraints/FKs; area containment; transaction rollback; indexes for measured queries |
| Redis integration | Ephemeral state | Location TTL; stale sample ignored; Redis restart/miss does not erase trip history |
| Concurrency | Race correctness | Two drivers accept one trip concurrently; exactly one winner; cancel vs accept race; duplicate quote consumption |
| Idempotency | Retry behavior | Same rider/key/payload returns one result; same key/different payload returns 409; retry after response loss does not duplicate payment/event |
| Security | Abuse/privacy | OTP enumeration/brute force; role escalation; IDOR; sensitive values absent from logs; location visible only to participants |

## CI checks required for code changes

Run from repo root with supported Rust stable and required services when applicable:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Database changes additionally require migration-up from an empty PostGIS database and focused integration tests. Do not test schema by applying an unreviewed migration to a database containing valuable data.

## Release checklist

- [ ] Every changed implementation slice maps to an approved US/AC.
- [ ] API contract, migration and data relationship docs match actual code.
- [ ] No secrets, `.env`, production PII, OTPs, tokens or precise locations are in Git.
- [ ] Authorization and concurrent booking claims have tests.
- [ ] Backup/restore, retention and incident procedures have owners before real users.
- [ ] Production provider, market, fares, privacy, SLO and security decisions are closed.
- [ ] Health/readiness behavior is verified with dependencies healthy and unavailable.
- [ ] Build, lint, tests and migration checks are green; unrun checks are stated plainly.

## Current state

The repo currently has only scaffold endpoints `/api/v1/health` and `/api/v1/ready`; P0 domain endpoint behavior remains to be implemented. See [`docs/api/openapi.yaml`](api/openapi.yaml) for the draft contract and [`docs/decisions/open-decisions.md`](decisions/open-decisions.md) for blockers.
