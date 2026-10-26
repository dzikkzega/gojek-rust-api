# ADR-0001 — Modular monolith and data ownership

- **Status:** Accepted for MVP
- **Date:** 2026-10-26
- **Decision owners:** Product/engineering owner (confirm owner names in project governance)

## Context

The MVP is an API-only ride-hailing backend for rider/driver trip flows. The project stack is explicitly agreed as Axum + Tokio, PostgreSQL + PostGIS, Redis, SQLx + Serde, and Tracing. Early traffic and team scale are not yet measured. Trip assignment must remain correct under concurrent requests, and current driver position is high-frequency/ephemeral.

## Decision

- Deploy one **modular monolith**. Organize Rust modules around domain boundaries as implementation needs them; do not split microservices in MVP.
- PostgreSQL is the durable source of truth for users, quotes, trips, status events, fare snapshots, cash-payment records, ratings and audit records.
- Redis stores expiring current driver location, cache, and short-lived coordination only. It is not authoritative for trip state, payment state, or exclusive assignment correctness.
- Perform booking, quote consumption, driver claim, trip state/event updates and completion/payment recording with explicit PostgreSQL transaction/conditional-write boundaries.
- Keep provider integrations (OTP, maps/routing, document storage, payment) undecided until their product/security decision is approved.

## Consequences

- Fewer deployment and operational components while domain behavior is validated.
- Database constraints and atomic SQL transitions are critical for concurrent matching correctness.
- Redis eviction/restart can reduce matching freshness temporarily but must not lose trip history.
- A future service split requires measured scaling/ownership evidence and an explicit ADR; do not prebuild it.

## Revisit when

Operational data shows independent scaling/release/security requirements that a modular monolith cannot meet, and an owner approves the added service/network/consistency cost.
