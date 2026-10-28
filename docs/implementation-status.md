# Documentation versus implementation status

This page prevents draft specifications from being mistaken for live API behavior. Update it whenever endpoint implementation ships.

## Implemented today

| Surface | Current behavior | Source |
|---|---|---|
| `GET /api/v1/health` | Liveness response; no dependency check | `src/main.rs` |
| `GET /api/v1/ready` | Checks configured PostgreSQL and Redis; missing/unavailable dependency returns 503. Reports SMTP configured/not configured. | `src/main.rs` |
| PostgreSQL/PostGIS local service | Compose service on loopback port 5433 by default; named volume; healthcheck | `compose.yaml` |
| Redis local service | Compose service on loopback port 6380 | `compose.yaml` |
| Mailpit local email sink | SMTP on loopback 1025; inbox UI on loopback 8025; does not deliver to real mailboxes | `compose.yaml` |
| PostGIS extension | First SQLx migration enables `postgis` | `migrations/20261025000100_enable_postgis.sql` |
| Email OTP identity migration | Adds normalized email identity; makes legacy phone optional; expires old phone challenges | `migrations/20261028000100_email_otp.sql` |
| OTP request | Sends 6-digit email through configured SMTP; stores hash and expiry. Does not reveal account existence. | `src/auth.rs` |
| OTP verify/session | Email-normalized lookup, attempt updates, single-use consume, rider/session creation and JWT response | `src/auth.rs` |
| Refresh/logout | Refresh rotation with row lock; revoke session | `src/auth.rs` |

## Designed, not implemented

- Profile, driver onboarding, quotes, matching/offers, trip lifecycle, tracking, cash payment records/read, ratings and P0 admin endpoint handlers in [`api/openapi.yaml`](api/openapi.yaml).
- `migrations/20261028000100_email_otp.sql` was applied to the local development DB during email-OTP conversion. Verify migration status in each environment before assuming it is applied there.
- Redis is available in Compose, but live driver location, candidate matching and offers are not yet implemented.
- OTP email sending works only with a configured SMTP relay. Mailpit is a free local sink; no production SMTP relay/sender domain is selected.
- Maps/routing, private document storage, push, payment provider, action-level admin permission matrix and production retention/security policies remain undecided or unimplemented.

## Before treating an operation as available

1. Implement its US/AC and route/handler.
2. Add unit/API/integration/security tests, including ownership and failure paths.
3. Apply reviewed migration(s) to a disposable PostGIS DB and verify with SQLx migration status.
4. Update OpenAPI examples/statuses to match actual implementation.
5. Move the operation from “designed” to “implemented” with evidence (tests/build/release reference).

Build/test status must be reported from commands actually run. `src/main.rs` requires Rust 1.94+ dependencies; the previous development environment did not have `cargo` available, so no build result is implied here.
