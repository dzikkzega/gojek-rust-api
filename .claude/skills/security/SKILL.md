---
name: security
description: Security requirements for authentication, user input, secrets, location data, admin actions, and external integrations.
---

# Backend Security Skill

## Trust boundaries
- Treat every request field, header, path ID, query parameter, uploaded file, and webhook as untrusted.
- Validate format, length, range, enum values, ownership, and allowed state transitions at the boundary.
- Use parameterized SQL. Never concatenate untrusted values into SQL, shell commands, file paths, or logs.
- Apply request-size limits, timeouts, and rate limits to public or abuse-prone operations.

## Authentication and authorization
- Require authentication for private resources. Check authorization and ownership on every request, not only in client code.
- Avoid account/phone enumeration in OTP request and verification responses.
- OTPs and credentials require expiry, attempt limits, rate limits, safe storage, and single-use/rotation behavior where specified.
- Sessions/tokens must be revocable. Never log or return token secrets after issuance.
- Never trust caller-supplied role, owner ID, trip status, fare, or payment state.

## Sensitive data
- Never commit secrets. Use environment-based local configuration and deployment secret storage; ignore local env files.
- Never log OTPs, passwords, access/refresh tokens, authorization headers, document contents, payment secrets, or precise coordinates.
- Minimize retention and exposure of phone, document, and location data. Show trip location only to authorized participants.
- Use synthetic test data. Do not copy production PII into fixtures, errors, or documentation.

## Admin, files, and webhooks
- Admin actions require least-privilege permissions and an audit record with actor, action, target, and timestamp; exclude secrets and unnecessary PII.
- File uploads require explicit allowlists, size limits, content validation, safe storage, and private access. Do not trust client filenames or MIME claims.
- Payment/provider webhooks require signature and replay validation, idempotent processing, and safe failure handling before being enabled.

## Review blockers
Block release for missing authorization, insecure secret handling, injection, OTP brute force/enumeration, location leakage, sensitive logs, or non-atomic trip assignment.
