# Product Requirements Document – Server App Manager

## 1. Executive Summary

Server App Manager is a single-tenant web platform that lets administrators and end-users deploy, manage, and monitor containerized applications through a centralized Angular UI and a Rust-backed REST API. The product wraps Docker / Docker Compose operations behind a role-protected, auditable API, and automates sub-domain provisioning, SSL termination, secrets management, backups, and rollbacks.

This document consolidates all current user stories into a coherent product scope, priorities, and acceptance criteria.

## 2. Product Vision

Provide a secure, reproducible, and user-friendly “app store” experience for self-hosting containerized applications, where a single click installs an app on its own sub-domain and a single click removes it cleanly.

## 3. Goals & Objectives

- **Reproducibility**: Every deployment is defined by a versioned YAML file and runs inside an identical Docker Compose stack.
- **Security**: JWT role-based access, Docker secrets, audit logging, CORS/CSRF protection, and hardened HTTP headers.
- **Automation**: One-click install, automatic health checking, rollback on failure, and Let’s Encrypt certificate provisioning.
- **Observability**: Real-time dashboard, live install status, log streaming, and structured error messages.
- **Operational safety**: Pre-install backups, nightly backups, retention policies, and dependency-aware uninstall.

## 4. Prioritization Framework

Requirements are prioritized using the MoSCoW method mapped to release tiers:

| Priority | MoSCoW           | Meaning                                                                    | Release Target |
| -------- | ---------------- | -------------------------------------------------------------------------- | -------------- |
| **P0**   | Must have        | Core functionality required for any usable release. Blocks MVP release.    | MVP            |
| **P1**   | Should have      | Important features that significantly improve usability or security.       | MVP / v1.0     |
| **P2**   | Could have       | Valuable additions that can be deferred without blocking core value.       | v1.1           |
| **P3**   | Won’t have (now) | Explicitly out of scope for the current release; reserved for future work. | Future         |

### Rationale

- **P0** covers the platform foundation, identity, authorization, and the core install/rollback/uninstall loop without which the product has no value.
- **P1** covers security hardening, auditability, admin controls, and validation required for production readiness.
- **P2** covers operational conveniences (secrets UI, SSL management, i18n, accessibility polish) that improve the experience but do not block an initial release.
- **P3** captures explicitly deferred capabilities from the user stories (true multi-tenancy, network policies, object storage backups).

### Prioritized Requirements Summary

| ID         | Requirement                                              | Priority  | Rationale                                                             |
| ---------- | -------------------------------------------------------- | --------- | --------------------------------------------------------------------- |
| FR-DP-1    | Reproducible Docker Compose Stack                        | **P0**    | The platform cannot run without a working baseline deployment.        |
| FR-AUTH-1  | Authentication & JWT                                     | **P0**    | All protected features depend on identity.                            |
| FR-AUTH-2  | Role-Based Permissions                                   | **P0**    | Required to distinguish end-user from admin capabilities.             |
| FR-APP-1   | App Store & One-Click Install Flow                       | **P0**    | Core product value proposition.                                       |
| FR-APP-2   | Configurable YAML App Definitions                        | **P0**    | Enables reusable, versioned app configurations.                       |
| FR-APP-3   | Install-Time Parameter Prompting                         | **P0**    | Required for configurable app installs.                               |
| FR-UNI-1   | Easy Uninstall                                           | **P0**    | Completes the core install/remove lifecycle.                          |
| FR-NET-1   | Per-App Network & Ingress                                | **P0**    | Required for sub-domain routing and app isolation.                    |
| FR-CL-2    | Container Health Dashboard                               | **P1**    | Critical for operations but the core loop works without it.           |
| FR-CL-1    | Launch Child Containers                                  | **P1**    | Admin power feature; app store covers primary use case.               |
| FR-CL-3    | Admin REST API for Containers                            | **P1**    | Needed for programmatic container management.                         |
| FR-UNI-2   | Admin-Only Removal with Dependency Guard                 | **P1**    | Safety feature for shared/dependent apps.                             |
| FR-AUTH-3  | Admin User Management                                    | **P1**    | Needed before non-admin users can be onboarded.                       |
| FR-ADMIN-1 | Admin App Management                                     | **P1**    | Enables admins to enable/disable/delete apps directly.                |
| FR-ADMIN-2 | Secrets Management                                       | **P1**    | Security-critical for production deployments.                         |
| FR-ADMIN-3 | Backup & Log Settings                                    | **P1**    | Operational safety and compliance.                                    |
| FR-RT-1    | Live Install Status                                      | **P1**    | Strongly improves UX during install.                                  |
| FR-RT-2    | Rate Limiting                                            | **P1**    | Prevents abuse of the install endpoint.                               |
| NF-SEC     | Security (JWT, secrets, CORS, CSRF, headers, audit)      | **P0/P1** | Core security controls are P0; some hardening can be P1.              |
| NF-PERF    | Performance & Reliability                                | **P1**    | Needed for production readiness.                                      |
| NF-OBS     | Observability                                            | **P1**    | Supports debugging and operations.                                    |
| NF-DEV     | Developer Experience (OpenAPI, validation, sanitization) | **P1**    | Required for maintainability and safe integrations.                   |
| NF-CI/CD   | CI/CD & Operations                                       | **P1**    | Required for reliable releases.                                       |
| FR-SSL-1   | SSL Certificate Management                               | **P2**    | Important for production but can run with manual/custom certs in MVP. |
| NF-A11Y    | Accessibility & Internationalization                     | **P2**    | Compliance and market expansion; does not block core value.           |
| —          | True Multi-Tenancy                                       | **P3**    | Explicitly deferred per user stories.                                 |
| —          | Cross-App Network Policies                               | **P3**    | Out of scope for current release.                                     |
| —          | Object-Storage Backups                                   | **P3**    | Path-configurable backup is sufficient for now.                       |

## 5. Target Users & Personas

| Persona              | Role                    | Primary Needs                                                                                                        |
| -------------------- | ----------------------- | -------------------------------------------------------------------------------------------------------------------- |
| **End-User**         | Authenticated user      | Browse app store, install/uninstall own apps, view My Apps, manage profile, see live install status.                 |
| **Administrator**    | `admin` role            | Full container management, user/role management, app enable/disable/delete, secrets management, SSL/backup settings. |
| **System Architect** | DevOps / platform owner | Reproducible Docker Compose stack, tenant namespacing, resource limits, CI/CD, security scanning.                    |
| **Security Analyst** | Compliance / audit      | JWT validation, audit logs, secret handling, security headers, CORS/CSRF controls.                                   |

## 6. Product Scope

### 6.1 In Scope

- Docker Compose baseline stack (Rust backend, Angular frontend, Traefik reverse proxy).
- Container lifecycle: launch, start, stop, inspect, logs, health monitoring.
- App store and one-click install/uninstall with dependency awareness.
- YAML-driven app definitions (`apps/store`, `apps/enabled`, `apps/disabled`).
- JWT authentication & role-based authorization (`user` / `admin`).
- Admin panels for apps, users, roles, secrets, backups, SSL, logs.
- OpenAPI 3.0+ spec served at `/docs` and `/docs-json`.
- Request schema validation & sanitization.
- Docker secrets management for sensitive values.
- Backup/retention configuration and automatic rollback.
- Single-tenant isolation with namespaced resources; multi-tenant support marked as future work.
- Let’s Encrypt and custom certificate management.
- Real-time status via SSE or WebSocket.
- Rate limiting on install operations.
- CORS, CSRF, security headers.
- CI/CD pipeline with tests, YAML linting, Docker build, security scan, and push to Docker Hub.
- WCAG 2.1 AA accessibility and English/Dutch internationalization.

### 6.2 Out of Scope

- True multi-tenant isolation (explicitly future work).
- Network policies for cross-app communication.
- Built-in object-storage backup backend (configurable path only).

## 7. Functional Requirements

### 7.1 Deployment Platform

**FR-DP-1 – Reproducible Docker Compose Stack [P0]**

- Repository contains a Dockerfile that builds the Rust backend (and optionally the Angular static server) into a single image.
- `docker-compose.yml` defines `backend`, `frontend`, and `traefik` services with ports 80/443, resource limits (`mem_limit`, `cpu_quota`), and the `app_data` volume mounted at `/data`.
- All Docker resources are prefixed with the current tenant ID.
- README documents build, start, and stop commands.

### 7.2 Container Lifecycle

**FR-CL-1 – Launch Child Containers [P1]**

- End-users can launch a container from the UI by providing image name, environment variables, port mappings, and optional command/entrypoint.
- Backend validates image existence, resource availability, port mappings, and command syntax before invoking Docker.
- Container is created with tenant network, tenant data volume, Docker secrets for secret env vars, user-provided `--env`, and the configured restart policy.

**FR-CL-2 – Container Health Dashboard [P1]**

- Dashboard lists containers with ID, image, status, health, CPU/memory averages, and last-checked timestamp.
- Table auto-refreshes every 10 seconds while preserving sort/filter state.
- Unhealthy rows are highlighted; tooltips show health-check details.
- Server-side logging of all health-check failures.
- Real-time updates via SSE or WebSocket.

**FR-CL-3 – Admin REST API for Containers [P1]**

- `POST /api/v1/containers/start/{containerId}`
- `POST /api/v1/containers/stop/{containerId}`
- `GET /api/v1/containers/{containerId}` (metadata)
- `GET /api/v1/containers/{containerId}/logs` (stream)
- All endpoints require valid JWT + `admin` role and enforce tenant ownership.

### 7.3 App Store & One-Click Installation

**FR-APP-1 – App Store & Install Flow [P0]**

- End-users see an **Install** button per app; clicking it opens a parameter modal.
- Backend retrieves the app YAML from `apps/store/<slug>.yaml` or a Git repo, substitutes placeholders, injects secrets, runs `docker compose up -d` / `docker run`, and waits up to 2 minutes for the health-check to pass.
- On success, the sub-domain is provisioned and the app appears in **My Apps** as Active.
- On failure, an automatic rollback is performed and a friendly error is shown.

**FR-APP-2 – Configurable YAML App Definitions [P0]**

- YAML follows a documented JSON Schema including image, service name, ports, env vars, volumes, dependencies, health-check, restart policy, etc.
- Variables are expressed as `{{PLACEHOLDER}}` and replaced at install time.
- Invalid YAML returns `400 Bad Request` with schema errors.
- Dependencies are resolved automatically during installation.

**FR-APP-3 – Install-Time Parameter Prompting [P0]**

- Modal shows mandatory fields and pre-fills optional fields with YAML defaults.
- Inputs are validated by type, regex, and length before submission.
- Backend substitutes values into rendered YAML or passes them as environment variables.
- Post-install summary card displays configured values.

### 7.4 Uninstall & Cleanup

**FR-UNI-1 – Easy Uninstall [P0]**

- End-users can uninstall an app from **My Apps** with a confirmation dialog.
- Backend stops/removes containers, optionally removes volumes/networks per YAML flag, and deletes the DNS/sub-domain entry.
- Dependent running containers trigger a warning and block uninstall unless explicitly handled.

**FR-UNI-2 – Admin-Only Removal with Dependency Guard [P1]**

- Non-admin users are blocked from uninstalling apps with running dependencies.
- Admins see a confirmation listing dependent containers; on confirm, dependents are stopped/removed first, then the target app, then cleanup.

### 7.5 Identity, Roles & Authorization

**FR-AUTH-1 – Authentication & JWT [P0]**

- `POST /auth/login` accepts email/password and returns a signed JWT in a Secure, HttpOnly, SameSite=Strict cookie.
- JWT contains `user_id`, `role`, and `tenant_id`.
- Protected endpoints verify the cookie, signature, role claim, and tenant ownership.
- Login and forgot-password endpoints are exempt from token validation.

**FR-AUTH-2 – Role-Based Permissions [P0]**

- Roles are defined in `roles.yaml` as named sets of permissions (e.g., `read:apps`, `write:containers`, `admin:all`).
- UI route guards only render admin routes for `admin` users.
- API endpoints that mutate apps, users, roles, or settings require `admin` role.
- Adding a new role only requires updating role-check logic.

**FR-AUTH-3 – Admin User Management [P1]**

- Admin panel lists users with username, email, role, status, last login.
- Create/edit/deactivate users with uniqueness checks, password policy, bcrypt hashing, and audit logging.
- Passwords are never returned in API responses.

### 7.6 Admin Configuration Panels

**FR-ADMIN-1 – App Management [P1]**

- Manage Apps section lists all apps (enabled/disabled).
- Actions: Edit Config, Enable, Disable, Delete.
- Enable creates container + sub-domain; Disable stops container and moves YAML to `disabled/`; Delete removes container, YAML, and sub-domain.

**FR-ADMIN-2 – Secrets Management [P1]**

- Secrets page supports create, list (masked), update, and delete of Docker secrets.
- Secrets are injected via `docker-compose.yml` `secrets:` entry as environment variables.
- Backend never logs secret values.

**FR-ADMIN-3 – Backup & Log Settings [P1]**

- Backup settings: retention days (default 30), max backups per day (default 7).
- Pre-install and nightly (02:00 UTC) backups of volumes and YAML directories.
- Log settings: retention days (default 90), max daily log size (default 10 MB).

### 7.7 Networking & Ingress

**FR-NET-1 – Per-App Network & Ingress [P0]**

- Each app gets a dedicated Docker network `app_<slug>`.
- The installer/admin container attaches to a special admin network spanning needed app networks.
- Traefik routes each app’s sub-domain to the correct container network.

### 7.8 SSL Certificates

**FR-SSL-1 – Certificate Management [P2]**

- Auto-SSL via Let’s Encrypt is ON by default.
- Admins can upload custom PEM cert/key stored as Docker secret `tls-custom`.
- Traefik router references the appropriate secret.
- UI shows certificate status and supports forced renewal.

### 7.9 Real-Time Status & Rate Limiting

**FR-RT-1 – Live Install Status [P1]**

- Install progress is pushed via SSE/WebSocket to all connected clients.
- UI shows spinner and transitions to Active or Failed with an error message.

**FR-RT-2 – Rate Limiting [P1]**

- Global: max 1 install per user per 30 seconds.
- Per-app: max 1 install per app per 5 seconds.
- Excess requests return `429 Too Many Requests`.

## 8. Non-Functional Requirements

### 8.1 Security [P0/P1]

- **JWT**: signed tokens in HttpOnly cookies; role and tenant claims enforced on every protected request.
- **Secrets**: Docker secrets for DB passwords, API keys, certificates; never returned to client or logged in plain text.
- **CORS**: only the Angular SPA origin is allowed.
- **CSRF**: token verified on POST/PUT/DELETE.
- **Headers**: CSP (nonce-based), X-Content-Type-Options, X-Frame-Options, Referrer-Policy, HSTS (HTTPS only), X-XSS-Protection applied by both Traefik and backend.
- **Audit logging**: every admin action and protected request logs user ID, endpoint, outcome, container/app ID, and timestamp.

### 8.2 Performance & Reliability [P1]

- Dashboard refreshes every 10 seconds without losing user sort/filter state.
- Install health-check timeout: 2 minutes.
- Automatic rollback on install failure.
- Docker resource limits per service.
- Rate limiting prevents install spam.

### 8.3 Observability [P1]

- Health endpoint per container: `GET /containers/{id}/health`.
- Container logs stream via `GET /containers/{id}/logs`.
- Daily plain-text logs in `/logs/<tenant>/` with rotation.
- Structured error responses with `code`, `message`, and optional `details`.

### 8.4 Accessibility & Internationalization [P2]

- WCAG 2.1 AA compliance: contrast, ARIA labels, keyboard focus order.
- English (`en.json`) and Dutch (`nl.json`) translation files using `ngx-translate`.
- All UI strings externalized; missing keys fall back to English.

### 8.5 Developer Experience [P1]

- OpenAPI 3.0+ spec at `api/swagger.yaml`.
- Served UI at `/docs` and raw spec at `/docs-json`.
- Versioned at `/api/v1`; spec validated in CI.
- JSON Schema validation with `cerberus` (or equivalent) for all mutable endpoints.
- Request bodies sanitized before Docker calls or file writes.

### 8.6 CI/CD & Operations [P1]

- GitHub Actions workflow (`.github/workflows/ci.yml`) with jobs: build/test, lint YAML, end-to-end tests, Docker build, security scan (Trivy/grype), push to Docker Hub.
- Triggers on push to `main` and pull requests.
- Caching for Cargo registry and Docker layers.
- Coverage reports uploaded as artifacts.

## 9. User Flows

### 9.1 End-User Installs an App

1. User browses App Store.
2. Clicks **Install** on an app; modal prompts for required parameters.
3. UI validates inputs and sends install request.
4. Backend takes pre-install backup, substitutes placeholders, creates container/network/sub-domain, polls health.
5. Real-time status updates show “Installing…”.
6. On success, app appears in **My Apps** as Active with sub-domain URL.
7. On failure, rollback runs and UI shows actionable error.

### 9.2 Admin Manages a Container

1. Admin opens Container Dashboard.
2. Selects a container and views health/resource metrics.
3. Uses REST API buttons (start/stop/inspect/logs) with tenant ownership enforced.
4. Backend audits the action and returns structured success/error response.

### 9.3 Admin Creates a User

1. Admin opens Users panel and clicks **Create User**.
2. Modal validates username/email uniqueness, password policy, and role selection.
3. Backend hashes password with bcrypt, stores user, writes audit log.
4. New user can log in with `user` or `admin` role.

## 10. Data & Entities

- **Tenant**: single-tenant now; tenant ID drives resource prefixing.
- **User**: `user_id`, `username`, `email`, `password_hash`, `role`, `status`, `last_login`.
- **App Definition**: YAML file in `apps/store`, `apps/enabled`, or `apps/disabled`.
- **Container**: Docker container linked to tenant and app slug.
- **Secret**: Docker secret name/value, referenced by YAML.
- **Backup**: timestamped archive of volume + YAML under `/backups/<tenant>/`.
- **Role**: permission mapping in `roles.yaml`.

## 11. API Requirements

- Base path: `/api/v1`.
- Authentication: JWT in HttpOnly cookie.
- Authorization: role claim enforcement.
- OpenAPI spec versioned and validated in CI.
- Error responses include `code`, `message`, optional `details`.
- Stateless endpoints except for SSE/WebSocket streams.

Key endpoint groups:

| Group      | Examples                                                                                                                                                                   |
| ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Auth       | `POST /auth/login`, `POST /auth/forgot-password`                                                                                                                           |
| Users      | `GET /users`, `POST /users`, `PUT /users/{id}/password`, `PUT /users/me`                                                                                                   |
| Apps       | `GET /apps`, `POST /apps/install`, `POST /apps/{slug}/uninstall`                                                                                                           |
| Containers | `POST /containers/launch`, `POST /containers/start/{id}`, `POST /containers/stop/{id}`, `GET /containers/{id}`, `GET /containers/{id}/health`, `GET /containers/{id}/logs` |
| Roles      | `GET /roles`, `POST /roles`, `PUT /roles/{name}`                                                                                                                           |
| Logs       | `GET /logs?appSlug=&date=`                                                                                                                                                 |
| Docs       | `GET /docs`, `GET /docs-json`                                                                                                                                              |

## 12. Security & Compliance Checklist

- [ ] JWT signing and validation implemented.
- [ ] HttpOnly Secure SameSite=Strict cookies.
- [ ] Role enforcement on UI routes and API endpoints.
- [ ] Docker secrets for sensitive env vars and certificates.
- [ ] CORS restricted to SPA origin.
- [ ] CSRF token verified on state-changing requests.
- [ ] Security headers on every response.
- [ ] Audit logging for all admin/protected actions.
- [ ] Secrets never returned or logged in plain text.
- [ ] YAML/schema validation before Docker or filesystem operations.
- [ ] Input sanitization for shell, YAML, and JSON contexts.

## 13. Open Issues & Future Work

- **[P3] Multi-tenancy**: true tenant isolation is deferred; current implementation uses namespacing and a TODO marker.
- **[P3] Network policies**: cross-app network isolation rules are out of scope.
- **[P3] Object storage backups**: currently path-configurable only.

## 14. Acceptance Criteria Summary

All acceptance criteria from `docs/USER_STORIES.md` are considered part of this PRD. The most critical criteria are:

| Priority | Criterion                                                                           |
| -------- | ----------------------------------------------------------------------------------- |
| **P0**   | Docker Compose baseline builds and runs cleanly on a fresh machine.                 |
| **P0**   | Only admin users can start/stop/inspect/launch containers and manage users/roles.   |
| **P0**   | One-click install succeeds within ~1 minute or cleanly rolls back within 2 minutes. |
| **P0**   | All state-changing requests are validated against JSON Schema and sanitized.        |
| **P0**   | Secrets, certificates, and passwords are never exposed to the client.               |
| **P1**   | OpenAPI spec is complete, versioned, and validated in CI.                           |
| **P1**   | CI/CD pipeline passes build, test, lint, security scan, and push jobs.              |
| **P2**   | UI meets WCAG 2.1 AA and supports English + Dutch.                                  |

## 15. Release Criteria

- All P0/P1 acceptance criteria pass.
- Security scan shows no CVE severity ≥ 7.
- OpenAPI spec passes `swagger-cli validate`.
- End-to-end install/uninstall flow verified in Docker-in-Docker CI.
- Accessibility audit passes automated checks.
- Documentation (README, YAML guide, API docs) is complete and accurate.
