# Security and data handling — MVP baseline

Engineering baseline, not legal advice or a complete privacy notice. Jurisdiction, lawful basis, retention periods, age requirements and data-subject processes remain open in [`../decisions/open-decisions.md`](../decisions/open-decisions.md). Do not onboard real users until accountable product/legal owners review them.

## Data inventory

| Data | System of record | Access | Baseline handling | Retention |
|---|---|---|---|---|
| Email/profile | PostgreSQL `users.email_normalized` / `email_verified_at` | Account owner; least-privilege staff | Trim/lowercase as currently implemented; generic auth errors; avoid logging address | TBD by jurisdiction |
| Phone/contact | Nullable legacy/contact field `users.phone_e164` | Account owner; least-privilege staff | Not used as the login identity for Email OTP; mask if used | TBD by jurisdiction |
| OTP challenge | PostgreSQL `auth_otp_challenges` | Auth service only | SHA-256 verifier bound to normalized email; one active challenge, expiry, attempt limit and cooldown; never log code | 5 minutes in current implementation; review with product |
| OTP email | SMTP relay; local Mailpit for development | Recipient inbox; SMTP credentials in app environment | TLS required for non-local SMTP; generic API response; never log OTP or full recipient | Mailpit is development-only; production relay retention TBD |
| Refresh token | Hash in PostgreSQL `auth_sessions` | Session service | High entropy, hash at rest, rotate/revoke; never log or return stored value | Session policy TBD |
| Driver documents | Private object storage; SQL stores metadata/key | Owner and permitted reviewers | Private ACL, allowlisted content/size, inspect files, random storage key; no public bucket | Legal retention/deletion TBD |
| Live driver location | Redis only | Matching service; authorized active-trip participants | Validate range/time; TTL/freshness and rate limits; redact precision in logs | TTL/post-trip removal TBD |
| Pickup/destination history | PostgreSQL/PostGIS `ride_quotes`, `trips` | Trip participants; limited operations staff | Object-level authorization; exclude from generic logs/analytics | Trip-history retention TBD |
| Fare/trip/cash record | PostgreSQL | Participants; authorized operations/finance | Integer minor units, ISO currency, immutable quote/fare snapshot | Accounting/legal policy TBD |
| Admin audit | PostgreSQL `audit_logs` | Authorized security/operations | Record actor/action/target/reason/request ID; exclude secrets and unnecessary PII | Audit retention TBD |

## Required controls

- TLS outside local development. Production secrets only in an approved secret manager; never commit production `.env` or credentials.
- OTP request/verify needs enumeration-resistant responses, per-email/network abuse controls, expiry, single-use and attempt limits. Non-local SMTP must require TLS; local development uses Mailpit. Numeric thresholds remain TBD.
- Check authentication and permission on every resource access. Rider owns own profile/quotes/history; driver may mutate only assigned trips; location is visible only to authorized trip participants.
- The DB `users.role` enum is not an action-level permission matrix. Define admin/CS permissions before exposing admin routes.
- Never trust client-provided owner ID, role, state, fare or payment status.
- Never log auth headers, tokens, OTP, full email/phone, document content/key, request bodies, precise coordinates or payment secrets. Redact before serialization.
- Audit sensitive staff reads as well as mutations when policy requires.

## Deletion, backups and incident readiness

- Do not promise hard deletion or retention intervals until legal/product decides how trip, payment, audit and regulatory records interact with account deletion.
- Define a process for access/correction/deletion requests before launch; determine which records may be erased or pseudonymized.
- Production backups contain the same sensitive data as the primary DB: encrypt, restrict access, define expiry, and test restores.
- Prepare credential rotation, session revocation, Redis-key purge, document-access revocation, incident escalation and evidence preservation procedures.

## Go-live blockers

Close D-05 through D-16, implement associated ACs, run auth/abuse/authorization tests, and obtain policy review before collecting real PII, location or identity documents.
