# TODO – Server App Manager

Project state tracked against `docs/PRD.md`, `docs/USER_STORIES.md`, and
`docs/ARCHITECTURE.md`. Priorities follow the PRD MoSCoW framework
(P0 = Must have / MVP, P1 = Should have, P2 = Could have, P3 = Deferred).

## Status legend

| Mark | Meaning |
| ---- | ------- |
| `[x]` | Done |
| `[~]` | Partial |
| `[ ]` | Missing / not started |

---

## P0 – Must have (blocks MVP)

- [x] **US-1 / FR-DP-1 – Reproducible Docker Compose stack**
  - Dockerfile, docker-compose.yml (backend, db, traefik), resource limits,
    named volumes, tenant prefixing, README commands.
  - Evidence: `backend/Dockerfile`, `docker-compose.yml`, `traefik/`, README.

- [~] **US-16 / FR-AUTH-1 – Authentication & JWT**
  - Login page, JWT in HttpOnly cookie, token validation on every request,
    password hashing (bcrypt).
  - State: crates present (`jsonwebtoken`, `bcrypt`); login endpoint returns
    `NotImplemented`; frontend login page is a placeholder.

- [~] **US-11,15 / FR-AUTH-2 – Role-based permissions**
  - Load `roles/roles.yaml` at startup, role-claim enforcement middleware,
    end-user vs admin gating.
  - State: `roles/roles.yaml` exists; no loader/middleware; route stubs return
    empty.

- [~] **US-7,14 / FR-APP-2 – Configurable YAML app definitions**
  - `apps/store`, `apps/enabled`, `apps/disabled` folders + YAML files,
    catalog loader, placeholder substitution, schema validation against
    `schemas/app-definition.schema.json`.
  - State: folders exist (empty); schema exists; no loader/YAML files.

- [ ] **US-6 / FR-APP-1 – One-click app install**
  - `POST /api/v1/apps/install`, dependency resolution, YAML render,
    `docker compose up`, sub-domain provisioning via Traefik.
  - State: endpoint stubbed `NotImplemented`.

- [ ] **US-8 / FR-APP-3 – Install-time parameter prompting**
  - Modal form collecting required/optional placeholders, validation, passing
    values to install engine.
  - State: no install modal in frontend.

- [ ] **US-9 / FR-UNI-1 – Easy uninstall**
  - One-click remove container + volumes + network + DNS cleanup.
  - State: endpoint stubbed `NotImplemented`.

- [~] **US-28 / FR-NET-1 – Per-app Docker network & ingress**
  - Each app in own network, backend reachable, Traefik sub-domain router.
  - State: Traefik dynamic config exists; no per-app network creation logic.

- [~] **US-5,26,32 / NF-SEC – Security core**
  - JWT validation, CORS, CSRF protection, security headers (CSP,
    X-Frame-Options, HSTS, Referrer-Policy).
  - State: CORS in `main.rs`; security headers in `traefik/dynamic.yml`; no
    CSRF; no JWT validation middleware.

---

## P1 – Should have (production readiness)

- [ ] **US-3 / FR-CL-2 – Container health dashboard**
  - Table (ID, image, status, health, CPU%, mem%, last checked), 10s
    auto-refresh preserving sort/filter, unhealthy rows red, SSE/WebSocket
    real-time updates, server-side failure logging.
  - State: no dashboard component; health endpoint returns static `unknown`.

- [ ] **US-2 / FR-CL-1 – Launch child containers from UI**
  - Form (image, env vars, port mappings, command override),
    `POST /containers/launch`, validation (image pull, resources, port regex,
    command syntax), Docker SDK run with network/volume/secrets/env/restart.
  - State: endpoint stubbed; no form.

- [~] **US-4 / FR-CL-3 – Admin container REST API**
  - Start, stop, inspect, logs endpoints (admin-only).
  - State: route stubs exist returning `NotImplemented`.

- [ ] **US-10 / FR-UNI-2 – Admin-only removal with dependency guard**
  - Block removal of apps with running dependents unless admin confirms.
  - State: missing.

- [~] **US-13 / FR-AUTH-3 – Admin user management**
  - View/create/edit/deactivate users from admin panel.
  - State: users route stubs exist; no admin UI.

- [~] **US-12 / FR-ADMIN-1 – Admin app management**
  - View/edit/enable/disable/delete apps incl. YAML + user values.
  - State: apps route stubs exist; no admin UI.

- [ ] **US-20 / FR-ADMIN-2 – Docker secrets management**
  - CRUD secrets, never expose values, admin panel UI.
  - State: no secrets routes/UI.

- [ ] **US-21,29 / FR-ADMIN-3 – Backup & log settings**
  - Pre-install + 02:00 UTC nightly backups, retention policy, daily
    plain-text logs with rotation + size limit, admin config UI.
  - State: no backup service; log endpoint returns empty.

- [ ] **US-25 / FR-RT-1, FR-RT-2 – Live install status + rate limiting**
  - SSE/WebSocket install progress, per-user/per-app install throttle.
  - State: no SSE; no rate limiter middleware.

- [ ] **US-22 – Rollback on install failure**
  - Revert container + YAML if health-check fails or any post-launch step
    fails.
  - State: missing.

- [~] **US-17,30 / NF-DEV – OpenAPI spec & API versioning**
  - Serve spec at `/docs` and `/docs-json`, `/api/v1/*` path versioning.
  - State: `api/swagger.yaml` complete & CI-validated; not served by backend.

- [~] **US-18,27 / NF-DEV – Schema validation, sanitization & friendly errors**
  - Validate state-affecting bodies vs JSON-Schema, sanitize, translate
    errors to actionable UI messages.
  - State: error types defined; no validation/sanitization middleware.

- [~] **US-19 / NF-CI/CD – CI/CD pipeline**
  - Tests, YAML lint, Docker build, security scan, Docker Hub push, e2e.
  - State: 6 jobs exist; e2e stubbed; no Docker Hub push.

- [~] **NF-OBS, NF-PERF – Observability & reliability**
  - Structured errors, log streaming API, health dashboard wiring, perf
    targets.
  - State: `tracing` crate present; log endpoint stubbed.

- [ ] **Audit logging**
  - Audit table (user_id, endpoint, outcome, resource_id, timestamp) +
    logging on all state-affecting calls.
  - State: no audit table in migrations; no logger code.

- [~] **Database schema completion**
  - Migrations for apps, containers, secrets, audit_logs, backups, settings.
  - State: only users + roles tables in `backend/migrations/0001_init.sql`.

---

## P2 – Could have (v1.1)

- [~] **US-24 / FR-SSL-1 – SSL certificate management**
  - Let's Encrypt auto + custom cert upload via admin UI.
  - State: ACME configured in Traefik; no custom cert upload UI.

- [~] **US-31 / NF-A11Y – Accessibility & i18n**
  - WCAG 2.1 AA compliance, English + Dutch translations.
  - State: `@ngx-translate` configured with `en`/`nl`; no a11y
    attributes/audit.

---

## P3 – Deferred (future work)

- [ ] **US-23 – True multi-tenancy** (resource namespacing beyond single
      tenant). Out of scope for current release.
- [ ] **Cross-app network policies**. Out of scope for current release.
- [ ] **Object-storage backups**. Path-configurable backup is sufficient for
      now.

---

## Suggested next focus

1. US-16 / FR-AUTH-1 – Auth & JWT (unblocks all protected endpoints)
2. US-11,15 / FR-AUTH-2 – RBAC enforcement (gates admin vs user)
3. US-7,14 / FR-APP-2 – App catalog loader + YAML definitions
4. US-6 / FR-APP-1 – One-click install (depends on auth + catalog)
5. US-9 / FR-UNI-1 – Easy uninstall (completes core lifecycle)
