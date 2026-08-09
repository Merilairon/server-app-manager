Below is a **technical‑constraints + edge‑case matrix** for every user story you supplied.  
I kept the format “**Story X – Technical Constraints**” followed by a bullet list of the concrete constraints that must be enforced, and then “**Story X – Edge Cases**” with the situations that commonly cause failures or require extra handling.

## Story 1 – Baseline Docker Container Setup

**Technical Constraints**

- Repository must contain a **Dockerfile** that builds the Rust backend (and optionally the Angular static server) into a single image.
- **docker‑compose.yml** must define three services: `backend`, `frontend`, `traefik`.
- Each service must expose **port 80 (HTTP)** and **port 443 (HTTPS)** (the Traefik router will terminate TLS).
- `docker compose up --build` must finish **without errors** on a clean machine (no pre‑built images, no cached layers that hide failures).
- Resource limits must be declared (`mem_limit`, `cpu_quota`) for each service (e.g., `mem_limit: 512m`, `cpu_quota: 50%`).
- A **named volume** `app_data` (scoped to the tenant) must be mounted into the backend container at `/data`.
- The **README** must include exact commands for building (`docker compose build`), starting (`docker compose up -d`), stopping/cleaning (`docker compose down`).
- All Docker resources (network, volume) must be **prefixed with the tenant ID** (`tenant_<id>_backend`, `tenant_<id>_frontend`, `tenant_<id>_data`, etc.).

**Edge Cases**

- Clean machine missing Docker or Docker‑Compose → build fails; need clear error message.
- Insufficient host resources (CPU/memory) to satisfy the declared limits → container start fails; validation should reject the compose file before `up`.
- Tenant‑ID collision (two tenants using the same ID) → networking/volume name clash; the system must enforce uniqueness (e.g., via CI check or runtime validation).
- File‑permission problems on the host (e.g., read‑only mount points) → volume mount fails; ensure the compose file uses appropriate `read_only: false` and that the host directory is writable.
- Image build failure (Rust compilation errors, missing dependencies) → `docker compose build` exits non‑zero; CI should surface the exact build log.

## Story 2 – Launch Child (Sub‑) Containers from the UI

**Technical Constraints**

- UI form must collect: image name/build context, optional key/value env vars, port mappings (`host:container`), optional command/entrypoint override.
- Backend endpoint `POST /containers/launch` must require a valid JWT (HttpOnly cookie) and verify the caller’s **role** (must be allowed to launch containers).
- Validation steps:
  1. Image existence (pull if missing).
  2. Resource availability (CPU/memory) for the requested container (use Docker SDK `inspect` or `docker system df`).
  3. Port‑mapping format validation (regex `^[0-9]+(:[0-9]+)?$`).
  4. Command syntax validation (ensure it’s a valid shell command or empty).
- Docker SDK call must include:
  - `--network tenant_<id>_net` (or create a new network if absent).
  - `--mount type=volume,source=tenant_<id>_data,target=/data` **only if** the image expects data (detect via image metadata or a flag).
  - `--secret` entries for any secret env vars referenced in the request.
  - `--env` for user‑provided variables.
  - `--restart unless-stopped` (or value from a default YAML if supplied).
- Response must return JSON with `container_id` and `"status":"Running"`.
- UI must show a **success toast** and refresh the **Container List** (status = Running).
- Error handling: translate Docker SDK errors into clear messages (e.g., “Image not found”, “Insufficient memory”).

**Edge Cases**

- Image name is misspelled or does not exist → Docker pull fails; backend should catch the error, return 400 with “Image not found – verify the name”.
- Port‑mapping conflict (host port already in use) → Docker returns 403; UI should display “Port already occupied”.
- Insufficient host resources → Docker returns 403 “could not allocate memory”; UI shows “Insufficient resources”.
- Invalid command syntax (e.g., contains shell metacharacters) → validation rejects before Docker call; UI shows “Command syntax invalid”.
- Network creation failure (e.g., network name already exists but is in use by another tenant) → backend must either reuse the existing network or return a clear error.
- Secret not found (referenced in request but secret does not exist) → 400 “Secret <name> not defined”.

## Story 3 – Container Health Dashboard

**Technical Constraints**

- Page “Container Dashboard” must render a table with columns: ID, Image, Status, Health, CPU % (10 s avg), Memory % (10 s avg), Last Checked (timestamp).
- Table must **auto‑refresh** every 10 seconds **without losing** current sort/filter state (use client‑side state persistence).
- Health endpoint `GET /containers/{id}/health` must call Docker’s built‑in health‑check if present; otherwise invoke the container’s `/healthz` endpoint.
- Rows with **Unhealthy** health must be highlighted in red and show a tooltip with the health‑check details (e.g., “failed – connection timeout”).
- Server‑Sent Events (SSE) or WebSocket must push health/status changes instantly while the page is open.
- Any health‑check failure must be logged server‑side with timestamp, container ID, and health status.

**Edge Cases**

- Docker health‑check not defined → fallback to `/healthz`; if that endpoint is unreachable, health status stays “Starting” until timeout.
- Network partition between UI and backend → SSE connection drops; UI should show a “reconnect” indicator and retry automatically.
- High CPU/Memory usage causing the 10‑second average calculation to lag → consider using a moving‑average window or throttling the refresh rate temporarily.
- Multiple health‑check failures in rapid succession → log each occurrence; UI should not flood with too many tooltips (debounce).
- Container stops after the page loads → health status changes to “Stopped”; UI must reflect the new status immediately.

## Story 4 – Container Management REST API (Admin only)

**Technical Constraints**

- All endpoints under `/api/v1/containers` require a **valid JWT** (HttpOnly cookie) and the caller must have the **Administrator** role.
- Endpoints:
  1. `POST /containers/start/{containerId}` → `docker start` → 200 OK, body `{ "containerId": "..."} `.
  2. `POST /containers/stop/{containerId}` → `docker stop` → 200 OK.
  3. `GET /containers/{containerId}` → metadata (ID, image, status, labels) → 200 OK.
  4. `GET /containers/{containerId}/logs` → stream logs (`docker logs -f`) → `text/event-stream` (or WebSocket).
- Validation: container must exist **and** belong to the caller’s tenant.
- Role enforcement: non‑admin → 403 Forbidden with “Access denied”.
- Docker‑related errors must be mapped to appropriate HTTP codes (404 if container not found, 400 for bad request, 500 for internal errors).
- OpenAPI spec must be stored in `api/swagger.yaml`, served at `/docs` (Swagger UI) and `/docs-json` (raw spec). CI must validate the spec.

**Edge Cases**

- Container ID does not exist → 404 Not Found.
- Container belongs to a different tenant → 403 Forbidden (or 404, depending on design).
- Docker daemon is unreachable (e.g., service restart) → 503 Service Unavailable; UI should surface “Service temporarily unavailable”.
- Log streaming stalls (e.g., no new log lines) → SSE should keep connection alive; UI must handle “no data” gracefully.
- Simultaneous start/stop requests for the same container → race condition; backend should serialize actions or return appropriate conflict (409).

## Story 5 – Security & Authorization for Container Management

**Technical Constraints**

- Angular UI components that trigger container actions must be **conditionally rendered** only when the JWT contains `admin` role (Angular guard).
- Backend must **re‑check** the JWT role on every container‑management request; non‑admin → 403 Forbidden.
- Secrets (Docker secrets) must never be included in any API response; only high‑level status fields are returned.
- Audit logging for each admin action: record user ID, endpoint, outcome (success/failure), container ID (if applicable).
- CORS policy: only `https://app.example.com` allowed; any other origin → 403 Forbidden.

**Edge Cases**

- Token expires during a long‑running request → request fails with 401; UI should redirect to login.
- JWT without `role` claim (malformed token) → 401 Unauthorized.
- Admin attempts to launch a container that belongs to a different tenant → 403 Forbidden.
- CORS misconfiguration (e.g., missing `Access-Control-Allow-Origin`) → browser blocks request; backend must still enforce 403 for non‑allowed origins.

## Story 6 – One‑Click App Installation

**Technical Constraints**

- UI “Install” button opens a modal that collects **required** parameters (e.g., DB name, API key) plus optional defaults.
- UI validation: non‑empty, correct format (e.g., email regex, password strength).
- Backend flow:
  1. Load app YAML from `apps/store/<slug>.yaml` (or Git repo).
  2. Substitute placeholders (`{{DB_PASSWORD}}`) with user‑provided values.
  3. Render a **Docker‑Compose snippet** (or direct `docker run` commands) and inject secrets via Docker secrets.
  4. Execute `docker compose up -d` (or equivalent) to create containers.
  5. Poll the container’s health endpoint (or Docker health‑status) for up to **2 minutes**; if not healthy, trigger rollback.
  6. Use Traefik API (or DNS provider) to create sub‑domain `app‑<slug>.example.com`.
- Progress UI: spinner + “Installing…” message while containers start.
- Success UI: “App is running”, sub‑domain URL displayed, app added to **My Apps** list with status **Active**.
- Failure UI: clear error (e.g., “Image not found”, “Insufficient memory”, “Health check timeout”) + backend logs failure with timestamp, user ID, container ID, error details.
- Rollback: if health‑check fails within 2 minutes, automatically `docker rm -f` the container, restore previous YAML version (if backup exists), return **409 Conflict** with friendly message.

**Edge Cases**

- User provides an invalid value (e.g., DB name contains illegal characters) → UI validation catches it before request; otherwise backend returns 400 with field‑specific error.
- Placeholder substitution fails (missing placeholder) → 400 “Placeholder {{X}} not defined in app YAML”.
- Docker‑Compose file generation error (syntax error) → 500 Internal Server Error with details.
- Health‑check never becomes healthy (e.g., app crashes on startup) → rollback after 2 min, UI shows “Installation Failed – Rolled back”.
- Traefik DNS challenge fails (rate‑limit, DNS propagation delay) → UI shows “Domain provisioning failed – try again later”.
- Insufficient host resources (CPU/memory) → 403 “Insufficient resources”.

## Story 7 – Configurable App Definitions via YAML

**Technical Constraints**

- YAML must conform to a **JSON‑Schema** that defines required fields (`image`, `service_name`, `ports`, `env`, `volumes`, `depends_on`, `healthcheck`, `restart_policy`, etc.).
- Storage locations: `apps/store/<slug>.yaml` (read‑only), `apps/enabled/<slug>.yaml`, `apps/disabled/<slug>.yaml`.
- Backend must **validate** the YAML against the schema; invalid files → 400 Bad Request with list of schema errors.
- Dependency resolution: if `depends_on: other-app`, the backend must ensure `other-app` is installed (create its containers first) before creating the dependent app.
- Placeholders (`{{PLACEHOLDER}}`) must be replaced with user‑provided values **before** rendering the compose file or invoking Docker.
- Documentation: `README_yaml.md` must explain schema, examples, placeholder usage.

**Edge Cases**

- YAML syntax error (e.g., missing colon) → schema validation fails → 400 with parsing error.
- Missing required field (e.g., `image`) → 400 “Missing required field ‘image’”.
- Invalid placeholder syntax (`{{` without `}}`) → 400 “Invalid placeholder syntax”.
- Dependency cycle (A depends on B, B depends on A) → detection during install; return 409 “Circular dependency detected”.
- YAML file stored outside allowed folders → 404 Not Found with clear message.

## Story 8 – Prompt for User‑Configurable Data at Install Time

**Technical Constraints**

- Modal lists **mandatory** fields from the YAML (marked `required`).
- Optional fields are pre‑filled with defaults defined in the YAML (e.g., `DB_HOST=db.example.com`).
- Each input must pass **type‑ and format‑validation** (e.g., regex for passwords, numeric ranges).
- Backend substitutes supplied values into the rendered YAML (or passes them as environment variables) before Docker launch.
- After installation, UI shows a **summary card** with the configured values for the user’s reference.

**Edge Cases**

- Mandatory field left empty → UI blocks submission; if submitted, backend returns 400 “Missing required field <name>”.
- Invalid format (e.g., email without `@`) → 400 “Invalid format for <field>”.
- Placeholder not present in YAML but user supplies a value → ignored (no error).
- User tries to install an app that references a secret that does not exist → backend returns 400 “Secret <name> not defined”.

## Story 9 – Easy Uninstall with Dependency Cleanup

**Technical Constraints**

- “My Apps” page shows an **Uninstall** button per app.
- Confirmation dialog asks “Are you sure you want to remove this app and its dependencies?” with **Cancel** / **Confirm**.
- Backend actions: stop/remove containers (`docker stop` / `docker rm`), optionally remove named volumes/networks (controlled by a flag in the YAML).
- DNS/sub‑domain removal via Traefik API or hosts‑file edit.
- Dependency guard: if dependent containers are running, UI shows warning and blocks uninstall unless user opts to also stop those dependencies.
- UI updates: app disappears from **My Apps** list; toast confirms “App uninstalled”.
- Audit log records: user ID, app slug, timestamp, list of removed resources.

**Edge Cases**

- Dependent containers are running → uninstall blocked; UI shows warning and disables **Confirm** until user explicitly opts to stop dependencies.
- Attempt to uninstall an app that has no YAML file (e.g., custom definition missing) → 404 Not Found.
- Volume removal fails (e.g., volume still in use by another container) → 500 Internal Server Error; UI shows generic “Could not remove volume”.
- DNS entry removal fails (Traefik API unreachable) → 502 Bad Gateway; UI shows “Failed to clean DNS”.

## Story 10 – Admin‑Only Removal with Dependency Guard

**Technical Constraints**

- Backend scans for running containers whose YAML lists the app as a dependency.
- Regular (non‑admin) users see a warning and cannot proceed; **Uninstall** button disabled or **Confirm** button disabled.
- Admin sees a confirmation dialog that enumerates dependent containers and asks explicit consent (“Stop and remove X, Y, Z and the app?”).
- Cleanup sequence: stop/remove dependent containers first, then the target app; then perform volume/network/DNS cleanup as in Story 9.
- UI feedback after success: “App and its dependencies have been removed”.
- Audit log records admin ID, app slug, list of removed containers/resources, timestamp.

**Edge Cases**

- Dependency detection misses a container (e.g., dynamic registration) → uninstall may leave stray containers; periodic reconciliation job recommended.
- Admin cancels the confirmation dialog → no action taken, UI remains unchanged.
- Dependent containers cannot be stopped (e.g., they hold critical data) → backend returns 403 “Cannot stop container <id>”.
- Concurrent uninstall requests for the same app → race condition; backend should serialize or return 409 Conflict.

## Story 11 – Role‑Based Identity & Authorization

**Technical Constraints**

- JWT must contain `user_id` and `role` claim (`user` or `admin`).
- Front‑end routing guard (`RoleGuardService`) checks token role on page load:
  - `user` → shows App Store, My Apps (read‑only), Profile.
  - `admin` → same plus Admin Panel with extra controls.
- API protection: endpoints that modify apps, users, roles, or system settings require `admin` role; otherwise 403 Forbidden.
- Logging: each role‑protected request logs user ID, endpoint, outcome.
- Extensibility: adding a new role only requires updating role‑check logic; UI logic stays unchanged.

**Edge Cases**

- Token missing or malformed → 401 Unauthorized; UI redirects to login.
- Token expires during a session → 401; UI forces re‑authentication.
- Role claim missing → 401; ensure token issuance always includes `role`.
- User with `admin` role tries to access a user‑only endpoint → 403 Forbidden.

## Story 12 – Admin Panel – App Management

**Technical Constraints**

- **Admin Panel** “Manage Apps” lists apps with columns: Name/slug, Status (Enabled/Disabled), YAML link (read‑only), Edit Config / Enable / Disable / Delete buttons.
- **Edit Config** modal allows modifying any user‑defined parameters; changes saved back to the YAML file in `enabled/` or `disabled/` folder.
- **Enable** → triggers container creation from the YAML, provisions sub‑domain via Traefik, updates status to **Enabled**.
- **Disable** → stops/removes container(s), moves YAML to `disabled/`, updates status to **Disabled**.
- **Delete** → removes container(s), deletes YAML from its folder; if the YAML originated from `store/` (custom definition) admin may choose to delete the definition from `store/` (explicit opt‑in).
- Role enforcement: only admins may invoke any of these actions; normal users receive 403.
- Audit log entry for each admin operation (user ID, action, app slug, timestamp, details).

**Edge Cases**

- Editing config while the app is running → changes are applied on next restart (or immediate if backend supports hot‑reload).
- Enable action fails (e.g., insufficient resources) → 403/400 with error; UI shows error toast.
- Delete action on an app that is currently **Enabled** → backend should stop container first; if fails, UI shows “Cannot delete running app”.
- Attempt to delete a YAML file that is referenced by another enabled app → 409 Conflict, UI warns about dependent apps.

## Story 13 – Admin Panel – User Management

**Technical Constraints**

- **Users list** displays Username, Email, Role, Status, Last Login.
- **Create User** form: username, email, password, role(s). Backend validates:
  - Username/email uniqueness.
  - Password policy (min length, complexity).
  - Stores password hash with bcrypt.
  - Assigns **user** role by default; admin can select **admin** during creation.
- **Edit User** allows changing email, role, or toggling active/inactive; persists changes and writes audit entry.
- **Deactivate User** sets status to **Disabled**; user cannot log in but data remains.
- Passwords never returned in API responses; only hashed values stored.
- All user‑management endpoints require **admin** role; others → 403.

**Edge Cases**

- Duplicate username/email during creation → 400 “Username/email already exists”.
- Password policy violation → 400 with details.
- Deactivating a user that has installed apps → apps remain functional but user can’t log in; audit logs the deactivation.
- Editing a user’s role from `user` to `admin` → must verify admin privileges (prevent privilege escalation).

## Story 14 – Application Folder Structure & YAML Handling

**Technical Constraints**

- Filesystem layout inside the container/host:

  ```
  apps/
  ├─ store/      (read‑only predefined YAML files)
  ├─ enabled/    (YAML for installed & active apps)
  └─ disabled/   (YAML for installed but disabled apps)
  ```

- Backend loads app definitions **only** from these three folders; any other path → 404 Not Found.
- Install flow: copy YAML from `store/` → `enabled/` (or `disabled/` on abort) and inject user values.
- Admin actions:
  - **Enable** → move file from `store/` → `enabled/` (or `disabled/` → `enabled/`).
  - **Disable** → move file from `enabled/` → `disabled/`.
  - **Delete** → remove file from its folder; optionally delete from `store/` if custom definition.
- UI shows physical path (`apps/enabled/<slug>.yaml`) in “App Details”.
- Validation: any request to load a YAML outside the allowed folders → 404 with clear message.

**Edge Cases**

- File permission errors on the host (e.g., read‑only `store/` directory) → backend cannot copy or delete; returns 500.
- Concurrent enable/disable operations on the same slug → race condition; implement file lock or atomic move.
- YAML file corrupted (invalid UTF‑8) → 400 “Invalid YAML”.
- Symlink attacks (pointing outside allowed folder) → filesystem restrictions (e.g., `chroot` or path sanitization) must prevent traversal.

## Story 15 – End‑User View – Secure App Browsing

**Technical Constraints**

- Front‑end **RoleGuardService** checks JWT role before rendering any route.
- Public routes (`/store`, `/my-apps`, `/profile`) are reachable by any authenticated user; UI controls are read‑only (no start/stop/delete).
- Admin routes (`/admin/...`) are only rendered when token contains `admin`; otherwise redirect to login or show “Not authorized”.
- UI elements that imply admin actions (buttons, menus) are hidden for normal users.
- All API calls include JWT; backend validates role before processing.

**Edge Cases**

- Token with `admin` role but UI still shows only user‑level UI → bug in guard logic.
- User manually crafts a request to an admin endpoint (e.g., via curl) → backend enforces role check → 403.
- Session timeout while on an admin page → redirect to login, losing context; UI should preserve intended destination via query param.

## Story 16 – Authentication, Authorization & Role‑Based Permissions

**Technical Constraints**

- **Login** endpoint `POST /auth/login` → on success redirects (302) to SPA; returns JWT via **Secure, HttpOnly, SameSite‑Strict** cookie.
- JWT signed (HS256 or RS256) with claims `user_id` and `role(s)`.
- **Token validation** on every protected endpoint (except `/auth/login` and `/auth/forgot-password`).
- **User registration** allowed only for users with `admin` role (admin UI “Create User” button).
- **Password change** endpoint `PUT /users/{id}/password` validates old password, enforces policy, updates bcrypt hash.
- **Profile management** via `PUT /users/me`; UI reflects changes instantly.
- **Role model**: roles are named sets of permissions (e.g., `read:apps`, `write:containers`, `admin:all`). Stored in `roles.yaml` (or similar) loaded at startup.
- **Admin role creation** via Admin Panel “Roles” section; definitions follow a JSON/YAML schema mapping permission names to booleans.

**Edge Cases**

- Registration without admin privilege → 403 Forbidden.
- Password change with weak password → 400 “Password does not meet policy”.
- Token replay attack (reusing old token) → JWT expiration and signature verification prevent it.
- Role claim missing or malformed → 401 Unauthorized.

## Story 17 – Swagger / OpenAPI Specification & Versioning

**Technical Constraints**

- `api/swagger.yaml` (YAML) contains the full OpenAPI 3.0+ document.
- All API endpoints are under base path `/api/v1` (future versions become `/api/v2`, etc.).
- Static serving:
  - `GET /docs` → Swagger UI (served from `dist/swagger-ui`).
  - `GET /docs-json` → raw spec (YAML/JSON) with CORS enabled for dev.
- Versioned docs: `/docs/v1`, `/docs-json/v1` (and later `/docs/v2`, etc.).
- CI pipeline runs `swagger-cli validate` (or `openapi-generator`) on `api/swagger.yaml`; build fails on validation errors.
- When a new major version is released, the `servers:` entry URL is updated and the spec version bumped.

**Edge Cases**

- Breaking change introduced without version bump → API compatibility broken; CI should catch mismatched server URLs.
- Spec file not found or unreadable → 500 Internal Server Error; ensure file exists in repo and CI copies it correctly.
- Documentation drift (code changes not reflected in spec) → CI validation will fail if endpoints are missing.

## Story 18 – Schema Validation & Sanitization

**Technical Constraints**

- Each mutable endpoint has an associated **JSON‑Schema** (e.g., `installRequestSchema`, `userCreateSchema`).
- Backend uses **cerberus** (or equivalent) to validate incoming payloads against the schema.
- Validation occurs **before** any Docker SDK call, file write, or DB operation.
- Invalid payload → **400 Bad Request** with list of field‑level errors (`field: message`).
- Sanitization before Docker commands or YAML rendering:
  - Shell‑escape command arguments.
  - Proper YAML quoting for placeholder substitution.
  - Escape special characters in JSON bodies.

**Edge Cases**

- Schema version mismatch (client sends data that violates new schema) → 400 with “Invalid request payload”.
- Extremely large payload (e.g., massive JSON) causing memory exhaustion → limit request size (e.g., 1 MB) and return 413 Payload Too Large.
- Malformed JSON (syntax error) → 400 “Invalid JSON”.
- Sanitization failure (e.g., command contains `; rm -rf /`) → sanitization step should reject or escape, preventing command injection.

## Story 19 – CI/CD Pipeline (GitHub Actions)

**Technical Constraints**

- Workflow file `.github/workflows/ci.yml` defines jobs: **build**, **lint‑yaml**, **test**, **docker‑build**, **security‑scan**, **push**.
- **build** job: `cargo build --release`, `cargo test` (unit + integration). Cache Cargo registry (`~/.cargo/registry`) and build artifacts.
- **lint‑yaml** job: runs `yamllint` + custom schema validator (`yq`/`ajv`) on all `*.yaml` files under `apps/`, `roles/`, config dirs.
- **test** job: starts Docker‑in‑Docker, runs end‑to‑end tests covering install/stop flows.
- **docker‑build** job: builds Docker image, tags with commit SHA and `latest`.
- **security‑scan** job: runs **Trivy** (or `grype`) on the built image; fails if any CVE severity ≥ 7 is found.
- **push** job: pushes image to Docker Hub using secrets `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN`.
- Triggers on `push` to `main` and on pull‑request events.
- Caching: `actions/cache` for Cargo registry and Docker layers.
- Artifacts: test coverage reports uploaded.
- Fail‑fast: any non‑zero exit code aborts pipeline.

**Edge Cases**

- Secrets missing or mis‑typed → push job fails; ensure CI secrets are correctly configured.
- Network restrictions preventing Docker‑in‑Docker (e.g., Docker daemon not available) → test job fails; consider using self‑hosted runners with Docker support.
- Lint failures due to outdated `yamllint` rules → update lint config or adjust files.
- Security scan false positives (e.g., known CVE that is not exploitable) → may need severity threshold tuning.

## Story 20 – Docker Secrets for Sensitive Environment Variables

**Technical Constraints**

- `docker‑compose.yml` includes a `secrets:` section per container, e.g.:

  ```yaml
  secrets:
    - source: db_password
      target: DB_PASSWORD
  ```

- **Admin Panel – Secrets page**:
  - **Create** → `docker secret create <name> <value>` (or Docker API).
  - **List** → shows secret names (masked values).
  - **Update** → `docker secret rm <name>` then `docker secret create <name> <new_value>`.
  - **Delete** → `docker secret rm <name>`; also remove reference from compose file.
- **Runtime usage** → Docker automatically injects secret as env var (`DB_PASSWORD`) inside the container.
- **No secret exposure** → backend logs never include secret values; only secret names (if needed).
- **Security** → secrets stored encrypted on Docker host; not baked into image layers.

**Edge Cases**

- Secret creation fails (e.g., value too large, invalid characters) → 400 with error details.
- Secret deletion while container is running → Docker prevents removal; backend must stop container first or return 409 Conflict.
- Secret name collision (two secrets with same name) → Docker enforces uniqueness; backend validates before creation.
- Secret value contains newline or special characters → Docker secret creation may reject; sanitize input.

## Story 21 – Backup Configuration & Retention

**Technical Constraints**

- Admin UI “Backup Settings” allows:
  - **Retention days** (integer ≥ 1, default 30).
  - **Max backups per day** (integer ≥ 1, default 7).
- **Pre‑install backup**: snapshot of app data volume, YAML file (`enabled/<slug>.yaml`) before container creation; stored under `/backups/<tenant>/` with timestamped filename.
- **Nightly backup** (cron at 02:00 UTC): creates timestamped backup of all persisted volumes and `enabled/`/`disabled/` YAML directories for the tenant.
- **Retention policy**: keep newest **N** files per day (N = _Max backups per day_); delete files older than _Retention days_ window.
- **Logging**: each backup operation logs start time, success/failure, duration, backup ID, and resources involved.
- **Storage**: backups stored on a mounted volume or configurable object storage (e.g., S3) via environment variable.

**Edge Cases**

- Insufficient disk space for backups → backup job fails; UI shows “Insufficient storage”.
- Permission errors when writing to backup directory → 500 error; ensure directory is writable.
- Concurrent backup and install operations → ensure atomicity (e.g., lock the app’s data directory).
- Backup of a volume that is being updated (file changes) → may capture inconsistent state; consider using volume snapshots (e.g., LVM) if available.

## Story 22 – Rollback Mechanism During Installation

**Technical Constraints**

- Backend records during container creation: container ID, YAML hash/timestamp, snapshot of data volume (if any).
- Health‑check monitoring: poll Docker health status (or readiness endpoint) for up to **2 minutes**.
- **Rollback trigger**: health not “healthy” within window → execute `docker rm -f <container_id>`.
- Restore previous YAML version (if backup exists) or re‑apply last known good state.
- Return **409 Conflict** with user‑friendly message: “Installation failed – automatic rollback performed.”
- UI shows spinner “Rollback in progress…”, then updates app status to “Installation Failed – Rolled back” and displays error toast.
- Audit log records: installer user, timestamp, container ID removed, YAML version restored.

**Edge Cases**

- Health‑check endpoint unreachable (network issue) → false positive; backend may treat as failure; consider multiple probes.
- Snapshot of data volume fails (e.g., storage error) → rollback may lose data; log the failure and still proceed with container removal.
- Concurrent install attempts for same app → race condition; backend should serialize or detect existing installation.
- Rollback itself fails (e.g., container cannot be removed) → return 500 and UI shows generic “Installation failed”.

## Story 23 – Single‑Tenant Isolation (Multi‑Tenant = TODO)

**Technical Constraints**

- Every request includes a **tenant identifier** (from JWT claim `tenant_id` or session variable).
- Docker resources (networks, volumes, backup directories) are **prefixed** with the tenant ID (`tenant_<id>_backend`, `tenant_<id>_data`, `/backups/tenant_<id>/`).
- UI shows current tenant in a dropdown (or hidden field); backend enforces that a user can act only on apps belonging to that tenant.
- Architecture is **future‑proof** for multi‑tenant: code uses tenant‑scoped names, no hard‑coded global names; adding true multi‑tenant (separate DB schemas, per‑tenant isolation) would require minimal changes.

**Edge Cases**

- Tenant ID missing or invalid → 400 Bad Request.
- Two tenants using the same Docker network name (e.g., `tenant_1_net`) → conflict; system must enforce uniqueness via naming scheme.
- Tenant‑specific resource leak (e.g., orphaned volume) → periodic cleanup job recommended.

## Story 24 – SSL Certificate Management (Let’s Encrypt & Custom)

**Technical Constraints**

- Admin UI “SSL Settings”:
  - **Auto‑SSL toggle** (default ON).
  - **Custom certificate upload** fields for PEM‑encoded cert and private key.
- **Auto‑SSL flow**: for each newly installed app, backend requests a Let’s Encrypt certificate for `app‑<slug>.example.com` using an ACME client (e.g., **cert‑manager** or **acme.sh**).
  - Obtained cert/key stored as Docker secret `tls‑cert`.
  - Traefik router configured to use this secret for HTTPS termination on the app’s sub‑domain.
- **Custom cert flow**: admin uploads PEM files → backend creates Docker secret `tls‑custom`; Traefik router updated to use this secret for the sub‑domain.
- **Certificate status** UI shows: valid, expiring within X days, error. “Renew” button forces Let’s Encrypt renewal.
- **Secret handling**: secrets mounted read‑only into Traefik; backend never logs certificate contents.
- **Fallback**: if Let’s Encrypt fails (rate‑limit, DNS challenge error), UI displays clear message and logs failure.

**Edge Cases**

- DNS challenge fails due to propagation delay → UI shows “Domain verification failed – check DNS records”.
- Rate‑limit exceeded (Let’s Encrypt) → UI informs user and schedules next renewal after 24 h.
- Custom certificate format invalid (missing PEM header) → 400 “Invalid certificate format”.
- Private key mismatch with certificate → Traefik logs TLS error; UI shows “Certificate/key mismatch”.

## Story 25 – Real‑Time Install Status & Rate Limiting

**Technical Constraints**

- **Rate limiting**:
  - Global: max **1** install request per user per **30 seconds**.
  - Per‑app: max **1** install request per app per **5 seconds**.
  - Excess requests → **429 Too Many Requests** with friendly message.
- **Install flow**: clicking **Install** sets app status to **“Installing…”** in **My Apps** list.
- Backend starts containers and begins health‑check polling (up to 2 min).
- **Live status**: backend pushes updates via **Server‑Sent Events (SSE)** or **WebSocket**; UI shows spinner + “Installing…”.
- **Completion**: on success → status → **“Active”**, sub‑domain URL shown, app added to **My Apps** with status **Active**.
- **Failure**: status → **“Failed”**, error toast/message displayed.
- **Audit**: each install attempt logged (user ID, app slug, start time, outcome).

**Edge Cases**

- User clicks Install repeatedly within 30 s → subsequent requests blocked with 429; UI shows “Please wait before trying again”.
- Network interruption during SSE/WebSocket → UI shows reconnection indicator and retries automatically.
- Health‑check never becomes healthy → after 2 min, rollback occurs (as in Story 22) and UI shows “Installation Failed – Rolled back”.
- Resource exhaustion (CPU/memory) during install → 403 “Insufficient resources”; UI shows appropriate error.

## Story 26 – CORS & CSRF Protection

**Technical Constraints**

- **CORS**: backend allows only origin `https://app.example.com`; all other origins receive **403 Forbidden**.
- **CSRF token**: SPA embeds token (e.g., meta tag or HttpOnly cookie). For state‑changing POST/PUT/DELETE endpoints, backend verifies token before processing.
- **Security headers** (added by middleware or Traefik):
  - `Content‑Security‑Policy` (nonce‑based for inline scripts).
  - `X‑Content‑Type‑Options: nosniff`
  - `X‑Frame‑Options: DENY`
  - `Referrer‑Policy: strict-origin-when-cross-origin`
  - `Strict‑Transport‑Security` (when HTTPS active).
  - `X‑XSS‑Protection: 1; mode=block` (legacy).
- Headers must be present on **every** response, including static assets.

**Edge Cases**

- Missing CORS header for a sub‑resource (e.g., API call to `/api/v1/containers/start`) → browser blocks request; backend must enforce CORS for all endpoints.
- CSRF token missing or outdated → 403 Forbidden; UI should refresh token before submitting forms.
- Header misconfiguration (e.g., CSP nonce not generated) → page may break; automated tests verify header presence.

## Story 27 – Graceful Error Handling & User‑Friendly Messages

**Technical Constraints**

- Every error response includes:
  1. `code` (e.g., `ERR_IMAGE_NOT_FOUND`).
  2. `message` (concise, user‑friendly).
  3. Optional `details` (actionable hint).
- Messages avoid technical jargon and suggest next steps (e.g., “Image not found – verify the image name”).
- Server‑side logging records timestamp, user ID, endpoint, and a **hash** of the payload (no secrets).
- Angular error interceptor catches HTTP errors, maps to appropriate toast/alert, and never displays raw stack traces to the user.

**Edge Cases**

- Unexpected server error (500) → generic “Something went wrong, please try again later” unless more specific code is set.
- Validation errors with many fields → list all errors in `details` array for clarity.
- Network timeout → 504 Gateway Timeout; UI shows “Request timed out, check your connection”.

## Story 28 – Per‑App Docker Network & Ingress

**Technical Constraints**

- When an app is launched, backend creates a dedicated Docker network named `app_<slug>` (e.g., `app_myapp`).
- The app’s container is attached to its network; the configuration/installer container is attached to an **admin** network that spans all app networks it needs to communicate with.
- Traefik router per app forwards traffic on the app’s sub‑domain (`app‑<slug>.example.com`) to the correct container network.
- Containers cannot directly communicate with other apps’ networks unless an explicit network‑policy rule is added (out of scope).

**Edge Cases**

- Network name collision (two apps with same slug) → backend must ensure uniqueness (e.g., append tenant ID).
- Traefik router misconfiguration (wrong host rule) → traffic may not reach the app; UI may show “App not reachable”.
- Network creation fails (e.g., driver not available) → 500 error; UI shows “Network creation failed”.

## Story 29 – Log Retention & Daily Plain‑Text Logs

**Technical Constraints**

- Admin UI “Log Settings”:
  - **Log retention (days)** – integer ≥ 1 (default 90).
  - **Max log size per day** – integer (bytes) (default 10 MB).
- **Daily rotation** at midnight: active log renamed to `app‑<slug>-YYYY‑MM‑DD.log`.
- **Size enforcement**: if a log exceeds max size, a new file is started for the same day (or file truncated, as configured).
- **Storage**: plain‑text logs in `/logs/<tenant>/`.
- **API**: `GET /logs?appSlug=…&date=YYYY-MM-DD` streams the requested day’s log.
- **Log entry format**: each line contains timestamp, log level, component, optional user ID.

**Edge Cases**

- Log file growth beyond disk quota → rotation may fail; monitor disk usage and alert admin.
- Timestamp format inconsistency → parsing errors in log viewer; enforce ISO‑8601.
- Simultaneous log writes from multiple processes → ensure atomic file write (e.g., append to separate buffer then rename).

## Story 30 – API Versioning

**Technical Constraints**

- All public APIs prefixed with `/api/v1` (future versions become `/api/v2`, etc.).
- Swagger spec lives at `/docs/v1` (UI) and `/docs-json/v1` (raw). Version number in spec matches URL.
- Backward‑compatible changes introduced in a new version; old versions stay supported for at least **12 months**.
- Deprecated endpoints are marked in Swagger UI and annotated in the spec (e.g., `deprecated: true`).

**Edge Cases**

- Old client still calling `/api/v1` after a major version bump → may break if non‑backward‑compatible changes were made; versioning must preserve compatibility.
- Documentation drift (spec not updated after code change) → CI validation will fail; ensure CI runs spec validation on every PR.

## Story 31 – Accessibility & Internationalization (i18n)

**Technical Constraints**

- WCAG‑2.1 AA compliance: sufficient colour contrast, ARIA labels, logical focus order, full keyboard operability.
- i18n implemented with **ngx‑translate** (or equivalent) using separate JSON files:
  - `en.json` – English strings.
  - `nl.json` – Dutch strings.
- Every UI string (buttons, error messages, placeholders, tooltips) sourced from translation files.
- Language selector (dropdown/flag) in top‑right; choice persisted via cookie or `localStorage`.
- Missing translation key → fallback to English string and log a warning for developers.

**Edge Cases**

- Inconsistent pluralization handling across languages → may cause UI glitches; test with various plural forms.
- Right‑to‑left languages (if added later) require layout adjustments; current implementation assumes LTR.
- Translation file missing a key → UI shows English placeholder; developer should be alerted.

## Story 32 – Security Headers & Hardening

**Technical Constraints**

- Middleware (Rust or Traefik) adds **security headers** to **every** response:
  1. `Content‑Security‑Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';` (adjust with nonces as needed).
  2. `X‑Content‑Type‑Options: nosniff`
  3. `X‑Frame‑Options: DENY`
  4. `Referrer‑Policy: strict-origin-when-cross-origin`
  5. `Strict-Transport-Security: max-age=31536000; includeSubDomains; preload` (only when HTTPS).
  6. `X‑XSS‑Protection: 1; mode=block` (legacy browsers).

- Headers are added both at the edge (Traefik) and by backend middleware; cannot be disabled via admin UI.
- **CSP nonce** for inline scripts/styles is generated per request, keeping policy tight while allowing necessary inline code.
- Automated tests verify that the headers exist on all routes, including static assets (CSS, JS, images).

**Edge Cases**

- Inline script without nonce → CSP violation → browser blocks script; UI may appear broken.
- HSTS header sent over HTTP → browsers may reject connection; ensure HSTS only on HTTPS responses.
- Header size limits (some proxies truncate headers) → verify that full header values are transmitted.
