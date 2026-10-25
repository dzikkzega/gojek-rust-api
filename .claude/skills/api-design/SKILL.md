---
name: api-design
description: REST API conventions for this Axum backend: resource URLs, HTTP semantics, errors, pagination, and versioning.
---

# API Design Skill

## Resource and versioning rules
- Use the PRD prefix `/api/v1`; use lowercase kebab-case nouns and plural resource names where suitable.
- Use standard HTTP semantics: GET reads, POST creates/actions, PUT replaces, PATCH partially updates, DELETE removes/revokes.
- Model domain commands (such as cancel/accept) as explicit actions only where plain CRUD does not fit.
- Keep breaking changes out of v1 unless compatibility is documented and approved.

## Responses and errors
- Use the PRD error envelope: `{ "error": { "code": "...", "message": "...", "request_id": "..." } }`.
- Use meaningful HTTP status codes: 400 malformed/invalid request, 401 unauthenticated, 403 unauthorized, 404 inaccessible/missing resource, 409 state/idempotency conflict, 422 domain rule violation, 429 rate limit, 500 unexpected server error.
- Never return stack traces, SQL errors, secrets, or sensitive resource details.

## Lists and writes
- Bound pagination; prefer cursor pagination for growing histories.
- Validate and normalize filters. Do not allow arbitrary sorting or field selection.
- Require idempotency for booking and other retry-sensitive writes as defined in the PRD.
- Return 201 for resource creation when appropriate; include stable resource identity.
- Document authentication, authorization, request fields, response fields, and expected errors in the task/API contract.

## Privacy and access
- Apply resource ownership checks for every nested or ID-based resource.
- Do not leak account existence from OTP request flows.
- Return live location only to authorized trip participants; do not expose driver-location search endpoints publicly.
