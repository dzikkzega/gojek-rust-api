# Documentation index

Recommended reading order:

1. [PRD](../PRD.md) — product scope, actors, MVP/P1 and acceptance gates.
2. [Task backlog](../TASK/TASKS.md) — User Stories and Acceptance Criteria.
3. [OpenAPI contract (draft)](api/openapi.yaml) — P0 request/response shapes and errors; unresolved choices remain TODO.
4. [Booking and matching flow](flows/booking-and-matching.md) — state machine, transaction boundaries, retry and failure cases.
5. [Visual ERD](database-erd.svg), [relationship notes](database-relationships.md), and [`migrations/`](../migrations/) — schema source of truth and its readable diagram.
6. [Open decisions](decisions/open-decisions.md) — product/operational choices not yet approved.
7. [Data handling/security baseline](security/data-handling.md) — sensitive data controls and go-live blockers.
8. [Architecture overview](architecture.md) — component ownership, transaction boundaries and current scaffold.
9. [Testing and release gates](testing-and-release.md) — test layers and go-live checklist.
10. [Implementation status](implementation-status.md) — implemented routes versus draft contract.
11. [Local development runbook](operations/local-development.md) — Docker PostgreSQL/PostGIS, SQLx migrations and API checks.
12. [Accepted ADR](decisions/ADR-0001-modular-monolith-and-data-ownership.md) — module and durable-data ownership decision.

## Sources of truth

- Product behavior/scope: `PRD.md` and approved US/AC cards.
- Database schema: forward-only SQLx migrations; ERD is explanatory, not authoritative.
- API behavior: OpenAPI draft, updated with implementation and AC changes.
- Production security, privacy, providers and numeric policies are not approved until their decision-register entries are closed.
