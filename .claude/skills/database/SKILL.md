---
name: database
description: SQLx/PostgreSQL/PostGIS and Redis data rules for the ride-hailing backend.
---

# Data Layer Skill

## PostgreSQL and SQLx
- Persist durable domain records in PostgreSQL; use SQLx parameter binding for every value.
- Migrations are the source-controlled schema history. Review migration safety and keep each migration narrowly scoped.
- Use database constraints for durable invariants: foreign keys, uniqueness, non-null requirements, and valid state where practical.
- Use transactions for related writes. Booking claims and driver assignment must have a database-enforced atomic boundary.
- Index observed query patterns, including foreign keys and status/time filters; avoid speculative indexes.
- Use `timestamptz` for instants and exact numeric/integer representation for money.
- Preserve quote/fare snapshots so tariff changes do not alter existing trips.
- Use PostGIS spatial types/functions only for requirements that need spatial queries; coordinate inputs must be validated.

## Redis
- Redis is for short-lived location, cache, and coordination; define TTLs and behavior after eviction/restart.
- Do not rely on Redis alone for trip state, payment records, authorization, or one-driver-per-trip correctness.
- Do not cache sensitive user data without an explicit requirement, TTL, and invalidation strategy.

## Query safety
- Never build SQL by concatenating untrusted strings. Bind values; whitelist dynamic identifiers/sort fields.
- Select only needed columns. Bound result sizes and set query/connection timeouts where appropriate.
- Treat database errors as internal diagnostics; return safe API errors and never leak SQL text to clients.
