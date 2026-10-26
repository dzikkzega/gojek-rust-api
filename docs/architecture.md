# Architecture overview — current scaffold and target MVP

**Status:** target architecture / work in progress. Only the health and readiness HTTP routes currently exist in `src/main.rs`; endpoint families in the OpenAPI draft are not implemented yet.

## Target request path

```text
Client
  → Axum route / extractor / boundary validation
  → domain command (auth, quote, matching, trip, payment, admin)
  → SQLx transaction + PostgreSQL/PostGIS for durable facts
  → Redis for expiring driver location / cache / short-lived offers
  → explicit response DTO / stable API error
```

One Rust process, one deployment unit: a modular monolith. Split modules by domain when their first accepted US/AC is implemented. Do not pre-create empty services or microservices.

## Component responsibilities

| Component | Owns | Must not own |
|---|---|---|
| Axum/API boundary | Routing, parsing, authentication extraction, input validation, status/error mapping | Fare authority, unchecked status mutation, authorization based only on client claims |
| Domain modules | State transitions, eligibility, quote/booking rules, authorization policy | SQL string construction or transport details scattered through business rules |
| PostgreSQL/PostGIS | Users, sessions, applications, vehicles, service areas, fare versions, quotes, trips, events, cash payment records, ratings, audit | Ephemeral high-frequency driver coordinates |
| Redis | Driver latest location with TTL, cache, short-lived matching offers/leases | Durable trip state, payment truth, or the sole protection against double assignment |
| Tracing | Request correlation, route/status/duration, dependency/error class | OTP, tokens, credentials, full phone, document payload/key, precise coordinates |

## Transaction boundaries

- **Booking:** verify owner/quote eligibility; atomically consume quote, create trip and initial event, persist idempotency result.
- **Driver acceptance:** conditional trip-state update and event in one transaction; driver/vehicle eligibility must be checked; exactly one winner.
- **Trip transition:** compare current state; update trip and append `trip_events` atomically.
- **Completion:** compute final fare server-side from approved versioned rule; update trip and cash-payment obligation in one transaction.
- **Rating:** verify completed trip and participant/direction; unique `(trip_id, direction)` protects duplicate rating.

Redis may optimize candidate fan-out but cannot make a durable transition authoritative. After Redis loss, DB facts remain valid; drivers may need to refresh location.

## Trust boundaries

1. Public requests: validate payload, rate-limit abuse-prone operations, return generic auth errors.
2. Authenticated rider/driver: verify ownership/participation for every object lookup/action.
3. Admin/CS: explicit permission per action; audit sensitive reads/writes; no universal unrestricted admin shortcut.
4. External providers (OTP, maps, document storage, payment): not selected yet. Add adapters only after corresponding decision is approved.

## Current scaffold

- `GET /api/v1/health`: liveness only.
- `GET /api/v1/ready`: checks configured PostgreSQL/Redis dependencies; currently reports not-configured as not ready.
- Startup reads process environment; it does not parse `.env` itself.
- Compose currently provisions PostgreSQL/PostGIS only; Redis is not in Compose.

## Module direction when implementing

Use domain modules such as `auth`, `users`, `drivers`, `vehicles`, `catalog`, `quotes`, `matching`, `trips`, `payments`, `ratings`, `admin`, `health`. Introduce each as required by an AC; keep SQL transactions and invariants explicit. See [PRD](../PRD.md), [OpenAPI draft](api/openapi.yaml) and [booking flow](flows/booking-and-matching.md).
