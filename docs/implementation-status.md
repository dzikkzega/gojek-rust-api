# Documentation versus implementation status

This page prevents draft specifications from being mistaken for live API behavior. Update it whenever endpoint implementation ships.

## Implemented today

| Surface | Current behavior | Source |
|---|---|---|
| `GET /api/v1/health` | Liveness response; no dependency check | `src/main.rs` |
| `GET /api/v1/ready` | Checks configured PostgreSQL and Redis; missing/unavailable dependency returns 503 | `src/main.rs` |
| PostgreSQL/PostGIS local service | Compose service on loopback port 5433 by default; named volume; healthcheck | `compose.yaml` |
| PostGIS extension | First SQLx migration enables `postgis` | `migrations/20261025000100_enable_postgis.sql` |

## Designed, not implemented

- All domain API operations in [`api/openapi.yaml`](api/openapi.yaml): auth/OTP, profiles, driver onboarding, quotes, matching, trip lifecycle, cash payment reads, ratings and P0 admin operations.
- The initial table schema in `migrations/20261025000200_initial_schema.sql` is a reviewed draft migration, not yet applied to the current development database unless `sqlx migrate run` has been run and verified.
- Redis is not part of the current Compose file; no driver-location/matching behavior is implemented.
- No OTP, routing/maps, private document storage, push, payment provider, authorization permission matrix, or production policy provider has been selected.

## Before treating an operation as available

1. Implement its US/AC and route/handler.
2. Add unit/API/integration/security tests, including ownership and failure paths.
3. Apply reviewed migration(s) to a disposable PostGIS DB and verify with SQLx migration status.
4. Update OpenAPI examples/statuses to match actual implementation.
5. Move the operation from “designed” to “implemented” with evidence (tests/build/release reference).

Build/test status must be reported from commands actually run. `src/main.rs` requires Rust 1.94+ dependencies; the previous development environment did not have `cargo` available, so no build result is implied here.
