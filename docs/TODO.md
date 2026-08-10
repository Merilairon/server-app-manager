# TODO – Server App Manager

Project state tracked against `docs/PRD.md`, `docs/USER_STORIES.md`, and
`docs/ARCHITECTURE.md`. Priorities follow the PRD MoSCoW framework
(P0 = Must have / MVP, P1 = Should have, P2 = Could have, P3 = Deferred).

User stories US-41–US-55 are the UI Design Implementation track derived from
`docs/DESIGN.md` and the mockups in `designs/`; each cross-references the
functional story it implements and inherits that story's priority.

## Status legend

| Mark  | Meaning               |
| ----- | --------------------- |
| `[x]` | Done                  |
| `[~]` | Partial               |
| `[ ]` | Missing / not started |

---

## P0 – Must have (blocks MVP)

- [x] **US-1 / FR-DP-1 – Reproducible Docker Compose stack**
  - Dockerfile, docker-compose.yml (backend, db, traefik), resource limits,
    named volumes, tenant prefixing, README commands.
  - Evidence: `backend/Dockerfile`, `docker-compose.yml`, `traefik/`, README.

- [x] **US-16 / FR-AUTH-1 – Authentication & JWT**
  - Login page, JWT in HttpOnly cookie, token validation on every request,
    password hashing (bcrypt).
  - Evidence: `backend/src/auth.rs`, `backend/src/middleware.rs`,
    `backend/src/routes/auth.rs`, `frontend/src/app/pages/login`.

- [x] **US-11,15 / FR-AUTH-2 – Role-based permissions**
  - Load `roles/roles.yaml` at startup, role-claim enforcement middleware,
    end-user vs admin gating.
  - Evidence: `backend/src/rbac.rs`, `backend/src/middleware.rs`,
    `backend/src/routes/apps.rs` `require_permission`.

- [~] **US-7,14 / FR-APP-2 – Configurable YAML app definitions**
  - `apps/store`, `apps/enabled`, `apps/disabled` folders + YAML files,
    catalog loader, placeholder substitution, schema validation against
    `schemas/app-definition.schema.json`.
  - Evidence: `backend/src/catalog.rs`, `apps/store/whoami.yaml`.
  - Missing: JSON-Schema validation; only `serde` deserialization is used.

- [x] **US-6 / FR-APP-1 – One-click app install**
  - `POST /api/v1/apps/install`, dependency resolution, YAML render,
    `docker compose up`, sub-domain provisioning via Traefik.
  - Evidence: `backend/src/install.rs`, `backend/src/routes/apps.rs`.

- [~] **US-8 / FR-APP-3 – Install-time parameter prompting**
  - Modal form collecting required/optional placeholders, validation, passing
    values to install engine.
  - Evidence: `frontend/src/app/components/install-modal`.
  - Missing: review step, SSE progress, N-ADOPT-1 backup checkbox.

- [x] **US-9 / FR-UNI-1 – Easy uninstall**
  - One-click remove container + volumes + network + DNS cleanup.
  - Evidence: `backend/src/uninstall.rs`, `frontend/src/app/pages/my-apps`.

- [x] **US-28 / FR-NET-1 – Per-app Docker network & ingress**
  - Each app in own network, backend reachable, Traefik sub-domain router.
  - Evidence: `backend/src/install.rs` compose generation with per-app
    `app_network` and `backend` networks, Traefik router labels.

- [x] **US-5,26,32 / NF-SEC – Security core**
  - JWT validation, CORS, CSRF protection, security headers (CSP,
    X-Frame-Options, HSTS, Referrer-Policy).
  - Evidence: `backend/src/middleware.rs` (JWT + CSRF),
    `backend/src/main.rs` `SetResponseHeaderLayer` (CSP, HSTS, etc.).

### P0 UI – MVP screens (implement P0 features)

- [x] **US-43 – Login Page UI** _(implements US-16 / FR-AUTH-1)_
  - Centered card on gradient backdrop, brand block, server-status badge,
    form submitting to `POST /api/v1/auth/login`, role-based redirect.
  - Evidence: `frontend/src/app/pages/login/login.{ts,html,scss}`.
  - Note: functional; server-status badge not yet implemented.

- [~] **US-45 – App Store Catalog UI** _(implements US-6 / FR-APP-1, US-15)_
  - Category filter chips, sort dropdown, responsive `.app-card` grid with
    install counts + social-proof line (N-ADOPT-2), Install action opens
    install modal.
  - Evidence: `frontend/src/app/pages/apps`.
  - Missing: category chips, sort dropdown, install counts/social-proof.

- [~] **US-46 – Install Modal UI** _(implements US-8 / FR-APP-3, US-22, US-25)_
  - Four-step modal (Configure → Review → Install → Health check), parameter
    validation, SSE live progress, post-install summary + N-ADOPT-1 backup
    commitment checkbox.
  - Evidence: `frontend/src/app/components/install-modal`.
  - Missing: review step, SSE progress, backup checkbox; health step is
    synchronous.

- [~] **US-47 – My Apps Lifecycle UI** _(implements US-9 / FR-UNI-1, US-25)_
  - Status filter chips, app cards with status badge / resource usage /
    lifecycle actions (Open, Pause, Restart, Uninstall), SSE real-time
    status, N-REL-2 dependency-aware uninstall confirmation.
  - Evidence: `frontend/src/app/pages/my-apps`.
  - Missing: status filters, resource usage, pause/restart, SSE.

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

### P1 – Behavioral Nudge Engine (FR-BNE-1/2/4)

- [ ] **US-33 / FR-BNE-1 – Nudge Engine Core Infrastructure**
  - Backend `NudgePolicy` module + `NudgeStore` (Postgres `nudges` &
    `nudge_prefs` tables via sqlx migrations), SSE delivery over the FR-RT-1
    channel with REST fallback (`GET /api/v1/nudges`), Angular `NudgeService`
    - `<sam-nudge>` component (banner / inline / toast slots using
      design-system tokens), 5 nudge API endpoints, audit integration,
      tenant-scoped. Phase 0 ships N-SEC-3 + N-REL-1.
  - State: `docs/NUDGE_ENGINE.md` spec complete; no backend module, no
    migrations, no SSE channel, no frontend service/component, no API
    endpoints. Blocked on FR-RT-1 SSE + audit logging.

- [ ] **US-34 – Nudge transparency, opt-out & ethical guardrails**
  - "Why am I seeing this?" panel, per-user "Reduce suggestions" opt-out,
    safety-snooze cap (`safety_snooze_max_hours`), no-dark-patterns +
    proportionality rules, per-user active-nudge rate cap (default 3),
    dismiss/snooze controls, audit trail for all lifecycle events.
  - State: missing; depends on US-33 core.

- [ ] **US-35 / FR-BNE-2 – Security domain nudges**
  - N-SEC-1 secret rotation reminder, N-SEC-2 strong-defaults on role
    assignment, N-SEC-3 loss-framed backup gap warning (Phase 0).
  - State: missing; N-SEC-3 is Phase 0 (P1), N-SEC-1/2 are Phase 1 (P1/P2).

- [ ] **US-36 / FR-BNE-2 – Reliability domain nudges**
  - N-REL-1 unhealthy container triage (Phase 0, safety), N-REL-2
    dependency-aware uninstall confirmation, N-REL-3 resource-limit
    suggestion on launch.
  - State: missing; N-REL-1 is Phase 0 (P1), N-REL-2/3 are Phase 1 (P1/P2).

- [ ] **US-39 / FR-BNE-4 – Nudge measurement & do-no-harm guardrails**
  - Daily offline metrics job (`nudge_metrics` table) rolling up per-nudge
    per-tenant metrics, four guardrails (24h uninstall rate, dismissal rate,
    safety action rate, audit volume < 5%), admin alerting on breach,
    `GET /api/v1/nudges/metrics` endpoint. Baseline metrics are P1; full
    per-nudge rollup dashboard is P2.
  - State: missing; depends on US-33.

### P1 UI – production screens (implement P1 features)

- [ ] **US-41 – Design System Foundation**
  - Migrate `designs/styles.css` tokens to `frontend/src/styles.scss` (bg,
    border, text, status tokens with `*-soft` variants), font stacks, base
    typography; implement shared standalone components under
    `frontend/src/app/shared/` (Card, StatTile, Button, Badge, Table,
    FormField, KeyValueRow, AppCard, Modal, ProgressSteps, ProgressBar,
    LogStream, Tabs, Avatar).
  - State: `designs/styles.css` + mockups exist; `styles.scss` is empty; no
    `shared/` directory or components. Foundational — blocks all UI stories.

- [ ] **US-42 – Application Shell & Navigation Model**
  - Sidebar (240px sticky) + topbar (56px) shell replacing the current
    top-nav, grouped navigation mirroring `roles/roles.yaml` (General +
    Administration), `AdminGuard` for admin routes, add routes `my-apps`,
    `containers`, `launch`, admin children (`admin/users`, `admin/apps`,
    `admin/secrets`, `admin/backups`, `admin/settings`).
  - State: `app.html` is a simple top-nav; routes exist only for `''`,
    `login`, `apps`, `admin`; no sidebar, no `AdminGuard`, no admin children.

- [~] **US-44 – Dashboard UI** _(implements US-3, US-21, audit, US-31)_
  - Four stat tiles (Installed Apps, Healthy, Unhealthy, Last Backup), My
    Apps preview, System Health bars (CPU/Mem/Disk/Network), Recent Activity
    audit feed with N-HYG-2 highlight, `<sam-nudge>` banner slot.
  - State: `pages/home` component exists; template is a welcome placeholder;
    no stat tiles, no preview, no health bars, no activity feed.

- [ ] **US-48 – Container Dashboard UI** _(implements US-3 / FR-CL-2, US-4)_
  - Topbar badges (auto-refresh 10s, SSE connected), stat tiles, container
    table with unhealthy row highlight, search + status filters, Launch
    button (admin), `?health=unhealthy` URL filtering, log stream panel.
  - State: no container dashboard page.

- [ ] **US-49 – Launch Container Form UI** _(implements US-2 / FR-CL-1)_
  - Two-column layout (form + validation/preview panels), Docker run fields,
    N-REL-3 resource-limit suggestion, pre-flight checks panel, live Compose
    preview.
  - State: no launch form component.

- [~] **US-50 – Admin Users & Roles UI** _(implements US-13 / FR-AUTH-3)_
  - Tabs (Users / Roles & permissions / Audit log), users table with avatar
    - role badge, Add/Edit dialog defaulting to `user` (N-SEC-2), roles
      reference table from `roles/roles.yaml`, filterable audit log.
  - State: `pages/admin` component exists; template is a placeholder; no
    tabs, no tables, no dialogs.

- [ ] **US-51 – Admin Apps & YAML Editor UI** _(implements US-12 / FR-ADMIN-1)_
  - Tabs (Catalog / Enabled / Disabled), apps table with N-ADOPT-3 stale
    version badge + "Diff & upgrade", inline YAML editor with schema
    validation, Discard/Validate/Save actions.
  - State: no admin-apps UI.

- [ ] **US-52 – Admin Secrets Management UI** _(implements US-20 / FR-ADMIN-2)_
  - Persistent warning banner, secrets table (name, scope, used-by, rotated),
    N-SEC-1 rotation reminder banner, New secret + Rotate modals, typed
    delete confirmation; values never exposed.
  - State: no secrets UI.

- [ ] **US-53 – Admin Backups & Logs UI** _(implements US-21 / FR-ADMIN-3, US-29)_
  - Tabs (Backups / Audit log / Live logs), stat tiles, backup history
    table with Restore/Download, live SSE log stream with severity colors.
  - State: no backups/logs UI.

- [ ] **US-54 – Admin Settings UI** _(implements US-24, US-21, US-25, NF-SEC)_
  - Tabs (General / SSL-TLS / Backups / Rate limiting / Security), SSL mode
    selection + custom cert upload, backup schedule, rate-limit fields,
    security header status grid + CSRF toggle, per-tab Save/Discard.
  - State: no settings UI.

---

## P2 – Could have (v1.1)

- [~] **US-24 / FR-SSL-1 – SSL certificate management**
  - Let's Encrypt auto + custom cert upload via admin UI.
  - State: ACME configured in Traefik; no custom cert upload UI.

- [~] **US-31 / NF-A11Y – Accessibility & i18n**
  - WCAG 2.1 AA compliance, English + Dutch translations.
  - State: `@ngx-translate` configured with `en`/`nl`; no a11y
    attributes/audit.

### P2 – Behavioral Nudge Engine (FR-BNE-3/4)

- [ ] **US-37 / FR-BNE-3 – Adoption domain nudges**
  - N-ADOPT-1 post-install backup commitment, N-ADOPT-2 app store
    social-proof refinement, N-ADOPT-3 stale app definition update nudge.
  - State: missing; Phase 2 set, gated on Phase 0/1 measurement baseline.

- [ ] **US-38 / FR-BNE-3 – Hygiene domain nudges**
  - N-HYG-1 empty-state onboarding nudge (starter apps + backup CTA),
    N-HYG-2 audit-log anomaly highlight.
  - State: missing; Phase 2 set, gated on Phase 0/1 measurement baseline.

- [ ] **US-39 (P2 portion) – Nudge per-nudge rollup dashboard**
  - Full per-nudge metrics visualization on `admin-settings.html` /
    `admin-backups.html` (baseline metrics are P1, see P1 section).
  - State: missing.

### P2 UI – accessibility & i18n compliance

- [~] **US-55 – Design Accessibility & i18n Compliance**
  - ARIA roles (navigation, tablist, dialog with focus trap), `aria-live`
    regions for log stream + SSE badges, visible focus styles, WCAG 2.1 AA
    contrast verification (axe-core in CI), full `@ngx-translate` string
    externalization (en/nl), skeleton loaders + empty/error states.
  - State: `@ngx-translate` configured with `en`/`nl` locales; no ARIA
    roles, no focus-trap, no focus styles, no axe-core CI, no skeleton
    loaders. Depends on US-41 design system + US-42 shell.

---

## P3 – Deferred (future work)

- [ ] **US-23 – True multi-tenancy** (resource namespacing beyond single
      tenant). Out of scope for current release.
- [ ] **Cross-app network policies**. Out of scope for current release.
- [ ] **Object-storage backups**. Path-configurable backup is sufficient for
      now.
- [ ] **US-40 / FR-BNE-5 – Nudge A/B Experimentation** (Phase 3). Variant
      assignment, sticky per-user allocation, guardrail-gated experiment
      lifecycle, per-variant metrics, admin Experiments UI. Gated on
      US-39 guardrails + sufficient tenant traffic.

---

## Suggested next focus

The critical path is the P0 MVP loop (auth → catalog → install → uninstall)
with the P0 UI screens wired to it. The design system + shell (US-41/US-42)
are the foundation for every UI story and should land early in parallel with
backend P0 work.

1. **US-16 / FR-AUTH-1** – Auth & JWT (unblocks all protected endpoints)
2. **US-11,15 / FR-AUTH-2** – RBAC enforcement (gates admin vs user)
3. **US-41 – Design System Foundation** + **US-42 – App Shell** (unblock all
   UI stories; replace top-nav with sidebar shell)
4. **US-7,14 / FR-APP-2** – App catalog loader + YAML definitions
5. **US-6 / FR-APP-1** + **US-45** + **US-46** – One-click install (backend
   engine + App Store catalog UI + install modal)
6. **US-9 / FR-UNI-1** + **US-47** – Easy uninstall (backend + My Apps UI)
7. **US-25 / FR-RT-1** – SSE channel (unblocks live install status + nudge
   delivery for US-33)
