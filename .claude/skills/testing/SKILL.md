---
name: testing
description: Focused testing approach for Rust backend acceptance criteria, API behavior, persistence, failure paths, and concurrency.
---

# Backend Testing Skill

## Test behavior
- Start from the assigned US/AC. Test observable behavior and domain invariants, not private implementation details.
- Prefer standard Rust `#[test]` and `#[tokio::test]`; add test dependencies only when a current test needs them.
- Keep tests deterministic, independent, and fast. Use synthetic data and explicit setup/cleanup.

## Minimum coverage by change
- Domain logic: valid and invalid inputs/states.
- HTTP endpoints: status, response shape, validation, authentication, authorization, and safe errors.
- Database: constraints, transaction rollback, migration compatibility, and ownership filters.
- Redis: TTL/freshness and behavior on miss, expiry, or outage.
- Retry-sensitive actions: same idempotency key, conflicting payload, and duplicate delivery.
- Matching/state changes: concurrent claims result in at most one assigned driver; invalid transitions do not mutate state.

## Verification
- Run targeted tests first, then `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` when possible.
- Report exact commands/results. If Cargo, toolchain, or services are unavailable, state that clearly; never claim tests passed without running them.
- Keep tests focused; do not build a broad test framework before needed.
