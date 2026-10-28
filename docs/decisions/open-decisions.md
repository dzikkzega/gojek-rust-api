# Open decisions before implementation / production

This is a decision register, not a source of invented defaults. `Open` means product/owner input is required. When closing an item, record the decision, owner, decision date (ISO 8601), rationale, affected US/AC, and links to updated docs. Never record secrets or personal data here.

| ID | Decision | Needed by | Status | Required outcome |
|---|---|---|---|---|
| D-01 | Launch city/region, service boundary and initial area dataset | US 05, US 10 | Open | Launch area(s), authoritative polygon source and rollout owner |
| D-02 | Currency, fare display and minor-unit convention | US 05, US 08 | Open | ISO currency, rounding and display rules |
| D-03 | Fare formula, route source, quote expiry and final-fare calculation | US 05, US 07, US 08 | Open | Versioned formula, route provider, final-amount authority |
| D-04 | Cancellation eligibility, reason codes, timing and fee policy | US 06, US 07 | Open | Actor/state matrix and fee handling; no fee before approval |
| D-05 | OTP provider/channel, TTL, resend, retry and attempt limits | US 01 | Open (blocking real sign-in) | Provider, region support, abuse controls, owner and cost limits. Current API deliberately returns 503 for OTP delivery until configured. |
| D-06 | Access/refresh token format, TTL, rotation and revocation | US 01 | Open | Token claims/storage, expiry, replay response and session limits |
| D-07 | Driver eligibility, identity/document types, age requirements and retention | US 03, US 10 | Open | Jurisdictional checklist, reviewer process, retention/deletion |
| D-08 | Private document storage, upload allowlist/size and content inspection | US 03 | Open | Storage provider, access, file policy, malware/content checks |
| D-09 | Matching radius, filters, offer order/fan-out, offer TTL and timeout | US 04, US 06 | Open | Deterministic MVP policy, tunables and retry/escalation behavior |
| D-10 | Location freshness, update frequency, Redis TTL and precision | US 04, US 07 | Open | Thresholds, expiry, consent, retention and authorized consumers |
| D-11 | Trip start-code generation, TTL, attempts and regeneration | US 07 | Open | Secure generation/hash, lifetime and brute-force controls |
| D-12 | Cash collection confirmation and discrepancy handling | US 08 | Open | Whether `paid` means driver attestation; reconciliation flow |
| D-13 | Languages, phone normalization and supported phone regions | US 01, US 02 | Open | Locales and numbering support |
| D-14 | Admin/CS permission model and emergency access | US 03, US 10 | Open | Least-privilege matrix, bootstrap and audit requirements |
| D-15 | Privacy notice, location consent, account deletion and retention | All | Open | Reviewed policy and jurisdictional obligations before real users |
| D-16 | SLO, peak load, latency, RPO/RTO and backup retention | US 10 / ops | Open | Numeric targets and tested recovery procedure |
| D-17 | Cursor format, page sizes, API error codes and request ID propagation | API contract | Open | Stable contract, defaults, bounds and examples |
| D-18 | Realtime transport and reconnect/resume semantics | US 07 | Open | Polling-only P0 or WebSocket, auth and event versioning |
| D-19 | Role model: one role per user or multiple role assignments | Schema / US 01, US 03 | Open | Confirm current `users.role` enum or approve migration to role table |
| D-20 | Non-cash provider, webhook verification, refunds and reconciliation | P1 payments | Deferred | Provider selection and idempotent ledger/reconciliation contract |

## Decision workflow

1. Assign an owner and decision deadline before dependent implementation begins.
2. Compare options and legal/operational effects; do not silently turn sample values into policy.
3. Update PRD, API contract, schema migration, ACs and tests in the same approved change.
4. For schema changes, add a forward migration. Never rewrite an already-applied migration.
5. Close the row with decision status/date/rationale and link to the decision record or approved spec.

## Current constraints

- No production provider, launch market, fare, cancellation fee, retention duration, TTL or numeric SLA is selected here.
- `.env.example` contains local-only Docker defaults; it is neither a production secret nor an approved deployment policy.
- Related docs: [`../api/openapi.yaml`](../api/openapi.yaml), [`../flows/booking-and-matching.md`](../flows/booking-and-matching.md), [`../../PRD.md`](../../PRD.md).
