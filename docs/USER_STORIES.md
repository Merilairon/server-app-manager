# User Story 1 – Baseline Docker Container Setup

As a system architect, I want a reproducible Docker Compose stack that defines the exact image, OS, runtime dependencies, environment variables, volume mounts, network settings, and default resource limits, so that the runtime environment is identical across development, testing, and production.

## Acceptance Criteria

- Repository contains:
  1. **Dockerfile** that builds the Rust backend (and optionally the Angular static server) into a single image.
  2. **docker‑compose.yml** that wires together:
     - `backend` service (Rust API)
     - `frontend` service (lightweight web server serving Angular files)
     - `traefik` service (reverse‑proxy)
  3. All services expose the required ports (80 HTTP, 443 HTTPS).
- **Build verification** – Running `docker compose up --build` on a clean machine finishes without errors and produces a running stack.
- **Resource limits** – The compose file sets sensible defaults (`mem_limit`, `cpu_quota`) for each service.
- **Volume mounts** – A named volume `app_data` (scoped to the current tenant) is mounted into the backend container at `/data`.
- **README** – Documents:
  - How to build the image (`docker compose build`).
  - How to start the stack (`docker compose up -d`).
  - How to stop/cleanup (`docker compose down`).
- **Tenant namespace** – All Docker resources (network, volume) are prefixed with the current tenant ID (`tenant_<id>_backend`, `tenant_<id>_frontend`, etc.).

## Technical Constraints & Edge Cases

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

# User Story 2 – Launch Child (Sub‑) Containers from the UI

As an end‑user, I want a “Launch Container” button with a form in the Angular UI, so that I can start additional workloads on demand.

## Acceptance Criteria

- **UI** – The form contains fields for:
  1. Docker image name or build context (text input).
  2. Optional environment variables (key/value pairs).
  3. Port mappings (`host:container`).
  4. Command/entrypoint override (optional).
- **API** – Clicking **Launch** sends a `POST /containers/launch` request with the form payload (JWT in cookie, role‑checked).
- **Backend validation** – The Rust service validates:
  - Image existence (pull if missing).
  - Sufficient resources (CPU/memory) for the requested container.
  - Correctness of port mappings and command syntax.
- **Docker SDK usage** – The backend runs `docker run` with:
  - `--network tenant_<id>_net` (or creates a new network if needed).
  - `--mount type=volume,source=tenant_<id>_data,target=/data` (if the image expects data).
  - `--secret` entries for any secret env vars referenced in the request.
  - `--env` for user‑provided variables.
  - `--restart unless-stopped` (or as defined in the YAML, if any).
- **Response** – Returns the new container ID and status “Running”.
- **UI feedback** – Shows a success toast and updates the **Container List** with the new container (status = Running).
- **Error handling** – If Docker returns an error (e.g., image not found, insufficient resources), the backend returns a clear error message and the UI displays it in a red banner.
- **Inspection** – The launched container appears in `docker ps` and can be inspected via `docker inspect <id>`.

## Technical Constraints & Edge Cases

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

# User Story 3 – Container Health Dashboard

As an end‑user or administrator, I want a Container Dashboard that lists all running containers with health, resource usage, and timestamps, so that I can quickly detect failures.

## Acceptance Criteria

- **Page** – “Container Dashboard” loads a table with columns:
  1. Container ID
  2. Image
  3. Status (Running / Paused / Stopped)
  4. Health (Healthy / Unhealthy / Starting)
  5. CPU % (last 10 s average)
  6. Memory % (last 10 s average)
  7. Last Checked (timestamp)
- **Refresh** – The table auto‑refreshes every 10 seconds without losing the current sort/filter state.
- **Health endpoint** – `GET /containers/{id}/health` returns the health status (uses Docker’s built‑in health‑check if present, otherwise a custom `/healthz` endpoint inside the container).
- **Visual cue** – Rows with **Unhealthy** health are highlighted in red; a tooltip shows the health‑check details (e.g., “failed – connection timeout”).
- **Logging** – Any health‑check failure is logged server‑side with timestamp, container ID, and health status.
- **Real‑time updates** – While the page is open, any change in health/status is reflected instantly (via Server‑Sent Events or WebSocket).

## Technical Constraints & Edge Cases

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

# User Story 4 – Container Management REST API (Admin only)

As an administrator, I want RESTful endpoints to start, stop, inspect, and retrieve logs of containers, so that I can automate container management tasks.

## Acceptance Criteria

- **Authentication** – All endpoints require a valid JWT (HttpOnly cookie) and the caller must have the **Administrator** role.
- **Endpoints** (all under `/api/v1/containers`):
  1. `POST /containers/start/{containerId}` – starts the container (`docker start`). Returns **200 OK** with container ID.
  2. `POST /containers/stop/{containerId}` – stops the container (`docker stop`). Returns **200 OK**.
  3. `GET /containers/{containerId}` – returns metadata (ID, image, status, labels). Returns **200 OK**.
  4. `GET /containers/{containerId}/logs` – streams logs as JSON‑encoded lines (`docker logs -f`). Returns **200 OK** with `text/event-stream` or WebSocket.
- **Validation** – Each request validates the container ID exists and belongs to the calling tenant.
- **Role enforcement** – Non‑admin callers receive **403 Forbidden** with a clear “Access denied” UI message.
- **Error mapping** – Docker‑related errors (e.g., container not found, permission denied) are translated to appropriate HTTP status codes (404, 400, 500) with user‑friendly messages.
- **OpenAPI spec** – The API is documented in `api/swagger.yaml` (versioned) and served at `/docs` (Swagger UI) and `/docs-json` (raw spec). The spec is version‑controlled and validated in CI.

## Technical Constraints & Edge Cases

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

# User Story 5 – Security & Authorization for Container Management

As an administrator, I need secure access controls so that only authorized users can launch, stop, or inspect containers.

## Acceptance Criteria

- **Angular guard** – UI components that perform container actions are only rendered when the JWT contains the `admin` role.
- **Backend enforcement** – Every container‑management endpoint checks the JWT’s `role` claim; non‑admin requests get **403 Forbidden**.
- **No secret leakage** – Environment variables that are secrets (read from Docker secrets) are never sent back to the client; only high‑level status fields are returned.
- **Audit logging** – Each admin action logs: user ID, endpoint, outcome (success/failure), and container ID (if applicable).
- **CORS** – Only the Angular SPA origin (`https://app.example.com`) is allowed; all other origins receive **403 Forbidden**.

## Technical Constraints & Edge Cases

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

# User Story 6 – One‑Click App Installation

As an end‑user, I want to press “Install” and have the app start automatically (≈ 1 min) on a sub‑domain, so that I get a ready‑to‑use instance without manual steps.

## Acceptance Criteria

- **UI** – “Install” button appears on the app store page. Clicking it opens a modal.
- **Modal** – Collects required configurable parameters (e.g., DB name, API key) plus optional defaults.
- **Validation** – UI validates each field (non‑empty, correct format) before submitting.
- **Backend flow**:
  1. Retrieves the app’s YAML definition (from `apps/store/<slug>.yaml` or a Git‑repo path).
  2. Substitutes user‑provided values (e.g., `{{DB_PASSWORD}}` → actual password).
  3. Generates a Docker‑Compose snippet for the app (or uses the YAML directly) and injects the secrets via Docker secrets.
  4. Calls `docker compose up -d` (or `docker run`) to create the container(s) and any dependent services.
  5. Waits (max 2 min) for the container’s health‑check to become healthy; polls the container’s readiness endpoint if needed.
  6. Uses Traefik’s API (or a DNS provider) to create the sub‑domain `app‑<slug>.example.com` (or updates the local hosts file for dev).
- **Progress UI** – Shows a spinner and “Installing…” message while the container is starting.
- **Success** – UI updates to “App is running”, displays the sub‑domain URL, and adds the app to the **My Apps** list with status **Active**.
- **Failure** – UI shows a clear error (e.g., “Image not found”, “Insufficient memory”, “Health check timeout”) and the backend logs the failure with timestamp, user ID, container ID, and error details.
- **Rollback** – If health‑check fails within 2 min, the backend automatically stops/removes the container and rolls back to the previous YAML version (if a backup exists) and returns **409 Conflict** with a friendly message.

## Technical Constraints & Edge Cases

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

# User Story 7 – Configurable App Definitions via YAML

As a product manager, I want each app to be described by a standardized YAML (or Docker‑Compose) file that can be stored locally or in Git, so that the full runtime configuration is version‑controlled and reusable.

## Acceptance Criteria

- **Schema** – The YAML follows a documented JSON‑Schema (image, service name, ports, env vars, volumes, dependencies, health‑check, restart policy, etc.).
- **Storage locations**:
  - **Local**: `apps/store/<slug>.yaml` (read‑only).
  - **Git**: backend can clone a repo and load `<repo>/apps/<slug>.yaml` at install time.
- **Validation** – The backend validates the YAML against the schema; invalid files cause a **400 Bad Request** with a list of schema errors.
- **Dependency resolution** – If an app references another app (e.g., `depends_on: db-app`), the backend resolves those dependencies automatically during installation (creates/containers first).
- **Placeholders** – Variables are expressed as `{{PLACEHOLDER}}`; the install flow replaces them with user‑provided values before rendering.
- **Documentation** – A `README_yaml.md` explains how to create or extend a YAML file, including examples of placeholders.

## Technical Constraints & Edge Cases

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

# User Story 8 – Prompt for User‑Configurable Data at Install Time

As an end‑user, I want the install flow to ask me for any configurable parameters defined in the app’s YAML, so that I can supply the required values before the container starts.

## Acceptance Criteria

- **Modal** – Lists all **mandatory** fields from the YAML (e.g., `DB_PASSWORD`, `API_KEY`).
- **Optional fields** – Pre‑filled with sensible defaults from the YAML (e.g., `DB_HOST=db.example.com`).
- **Validation** – Each input is validated (type, regex, length) before the request is sent.
- **Injection** – The backend substitutes the supplied values into the rendered YAML (or passes them as environment variables to Docker) before creating the container(s).
- **Summary** – After installation, the UI shows a **summary card** with the configured values for the user’s reference.

## Technical Constraints & Edge Cases

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

# User Story 9 – Easy Uninstall with Dependency Cleanup

As an end‑user, I want to uninstall an app with a single click so that the container and any associated resources are removed cleanly.

## Acceptance Criteria

- **My Apps page** – Shows an **Uninstall** button for each installed app.
- **Confirmation dialog** – “Are you sure you want to remove this app and its dependencies?” with **Cancel** / **Confirm** buttons.
- **Backend actions**:
  1. Stops and removes the app’s container(s) (`docker stop` / `docker rm`).
  2. Optionally removes named volumes or networks created for the app (controlled by a flag in the YAML).
  3. Deletes the DNS/sub‑domain entry (via Traefik API or hosts file edit).
- **Dependency guard** – If the app has running dependent containers, the UI shows a warning and blocks uninstall unless the user explicitly opts to also stop those dependencies.
- **UI update** – The app disappears from the **My Apps** list; a toast confirms “App uninstalled”.
- **Audit log** – Backend logs: user ID, app slug, timestamp, list of removed resources.

## Technical Constraints & Edge Cases

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

# User Story 10 – Admin‑Only Removal with Dependency Guard

As a product manager, I need to prevent regular users from removing an app if its dependency containers are still running, unless an admin explicitly confirms removal of both the app and its dependencies.

## Acceptance Criteria

- **Dependency detection** – Backend scans for any running containers whose YAML lists the app as a dependency.
- **Regular user flow** – If a non‑admin attempts uninstall on an app with running dependencies:
  - UI displays a warning: “This app has running dependencies. Uninstall is blocked for security reasons.”
  - The **Uninstall** button is disabled (or the dialog is shown with a disabled **Confirm**).
- **Admin flow** – Admin sees a confirmation dialog that lists the dependent containers and asks for explicit consent (“Stop and remove X, Y, Z and the app?”).
- **Cleanup** – Admin confirms → backend:
  1. Stops/removes dependent containers first.
  2. Then stops/removes the target app.
  3. Performs the same volume/network/DNS cleanup as in Story 9.
- **UI feedback** – After success, UI shows “App and its dependencies have been removed”.
- **Audit log** – Records: admin ID, app slug, list of removed containers/resources, timestamp.

## Technical Constraints & Edge Cases

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

# User Story 11 – Role‑Based Identity & Authorization

As a product manager, I want the application to enforce role‑based identity so that normal users can only view the app store, their installed apps, and their profile, while admins can access all admin‑authorized features.

## Acceptance Criteria

- **Token** – JWT includes `user_id` and `role` claim (`user` or `admin`).
- **Front‑end routing guard** – On page load, the SPA checks the token’s role:
  - **User** → shows **App Store**, **My Apps (read‑only)**, **Profile**.
  - **Admin** → shows the same plus **Admin Panel** with extra controls.
- **API protection** – Endpoints that modify apps, users, roles, or system settings require the `admin` role; otherwise they return **403 Forbidden**.
- **Logging** – Every role‑protected request logs user ID, endpoint, and outcome.
- **Extensibility** – Adding a new role (e.g., `audit`) only requires updating the role‑check logic; existing UI logic remains unchanged.

## Technical Constraints & Edge Cases

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

# User Story 12 – Admin Panel – App Management

As an administrator, I want a dedicated configuration panel where I can view, edit, enable/disable, and delete apps (including their YAML definitions and user‑provided values) so that I can manage the platform’s apps directly.

## Acceptance Criteria

- **Admin Panel** – “Manage Apps” section lists all apps (enabled & disabled) with columns:
  1. Name & slug
  2. Status (Enabled / Disabled)
  3. Link to YAML (read‑only)
  4. Buttons: **Edit Config**, **Enable**, **Disable**, **Delete**.
- **Edit Config** – Modal lets the admin modify any user‑defined parameters (environment variables, secrets). Changes are saved back to the YAML file in the appropriate folder (`enabled/` or `disabled/`).
- **Enable** – Triggers container creation from the YAML, provisions the sub‑domain via Traefik, and updates the app status to **Enabled**.
- **Disable** – Stops and removes the container(s), moves the YAML file to `disabled/`, and updates status to **Disabled**.
- **Delete** – Removes the container(s), deletes the YAML file from its folder, and removes the sub‑domain entry. If the YAML was originally from `store/` (i.e., a custom definition), it is also deleted from `store/` (only if the admin explicitly chooses to remove the definition).
- **Role enforcement** – Only admins can invoke any of these actions; normal users receive **403 Forbidden**.
- **Audit log** – Every admin operation logs: user ID, action, app slug, timestamp, and details (e.g., changed env vars).

## Technical Constraints & Edge Cases

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

# User Story 13 – Admin Panel – User Management

As an administrator, I want a user management panel where I can view, create, edit, and deactivate user accounts, ensuring that only authorized personnel can access the system.

## Acceptance Criteria

- **Users list** – Shows Username, Email, Role, Status (Active/Disabled), Last Login.
- **Create User** – Form with username, email, password, role(s). Backend:
  - Validates uniqueness of username/email.
  - Enforces password policy (min length, complexity).
  - Stores password hash (bcrypt).
  - Assigns the **user** role by default; admin can select **admin** during creation.
- **Edit User** – Allows changing email, role, or toggling active/inactive. Persists changes and writes an audit entry.
- **Deactivate User** – Sets status to **Disabled**; user can no longer log in, but data (profile, installed apps) is retained.
- **Security** – Passwords are never returned in API responses; only hashed values are stored.
- **Role‑based API** – All user‑management endpoints require the **admin** role; others receive **403**.

## Technical Constraints & Edge Cases

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

# User Story 14 – Application Folder Structure & YAML Handling

As a product manager, I want the application to organize its app definitions in a dedicated “apps” folder with sub‑folders **store**, **enabled**, and **disabled**, so that all predefined, installed‑enabled, and installed‑disabled YAML files are clearly separated and easily manageable.

## Acceptance Criteria

- **Filesystem layout** (inside the container or on the host):

  ```
  apps/
  ├─ store/      ← predefined YAML files for every available app (read‑only)
  ├─ enabled/    ← YAML files for apps that are installed & active (includes user‑defined values)
  └─ disabled/   ← YAML files for apps that are installed but currently disabled (includes user‑defined values)
  ```

- **Backend loading** – The service reads app definitions **only** from these three folders; any attempt to load a file outside this structure returns **404 Not Found** with a clear error message.
- **Install flow** – When an app is installed, the corresponding YAML file is copied from `store/` to `enabled/` (or `disabled/` if the install is aborted) and any user‑provided values are injected.
- **Admin actions**:
  - **Enable** moves the file from `store/` → `enabled/` (or from `disabled/` → `enabled/`).
  - **Disable** moves the file from `enabled/` → `disabled/`.
  - **Delete** removes the file from its folder (and optionally from `store/` if it was a custom definition).
- **UI display** – The “App Details” view shows the physical path (`apps/enabled/<slug>.yaml`) so admins can see where the file lives.
- **Validation** – The backend validates that any requested YAML file exists inside the allowed folders; attempts to load files outside this structure return **404 Not Found** with a clear error message.

## Technical Constraints & Edge Cases

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

# User Story 15 – End‑User View – Secure App Browsing

As an end‑user, I want to see only the apps I am allowed to view (my installed apps, the public app store, and my profile) and be prevented from accessing admin‑only pages or modifying admin resources.

## Acceptance Criteria

- **Route guarding** – A front‑end **RoleGuardService** checks the JWT’s role before rendering a route.
- **Public routes** (`/store`, `/my-apps`, `/profile`) are accessible to any authenticated user; they are read‑only (no start/stop/delete controls).
- **Admin routes** (`/admin/manage-apps`, `/admin/users`, etc.) are only rendered when the token contains `admin`; otherwise the user is redirected to the login page or shown a “Not authorized” message.
- **UI elements** – Buttons/controls that imply administrative actions (e.g., “Start”, “Stop”, “Delete”) are hidden for normal users.
- **API validation** – All API calls include the JWT; the backend validates the role before processing the request.

## Technical Constraints & Edge Cases

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

# User Story 16 – Authentication, Authorization & Role‑Based Permissions

As a security analyst, I want a login page that authenticates users via the Rust client, returns a JWT stored in an HttpOnly cookie, and validates that token on every request (except login/forgot‑password).

## Acceptance Criteria

- **Login endpoint** – `POST /auth/login` accepts `{email, password}`; on success it redirects (302) to the SPA. On failure returns **401** with a clear error.
- **JWT creation** – On successful login, the backend creates a signed JWT (HS256 or RS256) containing `user_id` and `role(s)`, and sends it to the client via a **Secure, HttpOnly, SameSite‑Strict** cookie.
- **Token validation** – Every protected endpoint (`/apps/*`, `/containers/*`, `/users/*`, `/roles/*`, etc.) verifies the cookie, checks the signature, extracts the role claim, and enforces the permission matrix.
- **Exemptions** – `/auth/login` and `/auth/forgot-password` are exempt from token validation; any request containing a valid token to these endpoints receives **401**.
- **User registration** – Only users with the **admin** role can create new accounts. The admin UI shows a “Create User” button that opens a modal with fields: username, email, password, role(s).
- **Password change** – Authenticated users can call `PUT /users/{id}/password` with `{oldPassword, newPassword, confirmPassword}`; the backend validates the old password, enforces policy, and updates the hash.
- **Profile management** – Users can edit their email and preferences via `PUT /users/me`; UI reflects changes instantly.
- **Role model** – A role is a named set of permissions (e.g., `read:apps`, `write:containers`, `admin:all`). The mapping is stored in `roles.yaml` (or similar) read at startup.
- **Admin role creation** – The Admin Panel includes a **Roles** section where an admin can create, edit, and delete roles. Role definitions use a JSON/YAML schema mapping permission names to booleans.

## Technical Constraints & Edge Cases

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

# User Story 17 – Swagger / OpenAPI Specification & Versioning

As a product manager, I need a versioned OpenAPI (Swagger) spec that is served locally as both a UI and a JSON file (`/docs` and `/docs-json`).

## Acceptance Criteria

- **Spec location** – `api/swagger.yaml` (YAML) contains the full OpenAPI 3.0+ document.
- **Versioning** – The base path for all API endpoints is `/api/v1` (future versions become `/api/v2`, etc.). The spec reflects this (`servers:` entry with `url: http://localhost/api/v1`).
- **Static serving**:
  - `GET /docs` → interactive Swagger UI (served from `dist/swagger-ui`).
  - `GET /docs-json` → raw spec (YAML/JSON) with CORS enabled for local development.
- **Endpoint coverage** – All API endpoints (auth, users, apps, containers, backup, logs, etc.) are documented with request/response schemas.
- **CI validation** – The CI pipeline runs `swagger-cli validate` (or `openapi-generator`) on `api/swagger.yaml`; the build fails if the spec is malformed.
- **Versioned docs** – The UI and JSON are also versioned (`/docs/v1`, `/docs-json/v1`) and the spec is bumped when a new major version is released.

## Technical Constraints & Edge Cases

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

# User Story 18 – Schema Validation & Sanitization

As a developer, I want all request bodies that affect state (install, user management, role management, container launch, etc.) to be validated against a JSON‑Schema and sanitized before use.

## Acceptance Criteria

- **JSON‑Schema** – Each mutable endpoint has an associated JSON‑Schema (e.g., `installRequestSchema`, `userCreateSchema`, `containerLaunchSchema`).
- **Validation library** – The Rust backend uses **`cerberus`** (or similar) to validate incoming payloads against the schema.
- **Error response** – Invalid payloads return **400 Bad Request** with a list of validation errors (`field: message`).
- **Sanitization** – Before any Docker SDK call or file write, user‑supplied strings that will be rendered into YAML or passed as command arguments are escaped:
  - Shell‑escape for command lines.
  - Proper YAML quoting for variable placeholders.
  - Escape special characters in JSON bodies.
- **Execution order** – Validation occurs **before** any Docker SDK invocation, file system operation, or database write.

## Technical Constraints & Edge Cases

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

# User Story 19 – CI/CD Pipeline (GitHub Actions)

As a DevOps engineer, I want a GitHub Actions pipeline that runs unit & integration tests, validates all YAML files (including roles and app definitions), builds the Docker image, runs a security scan, and pushes the image to Docker Hub.

## Acceptance Criteria

- **Workflow file** – `.github/workflows/ci.yml` defines the following jobs:
  1. **build** – `cargo build --release`, runs `cargo test` (unit + integration). Caches Cargo registry and build artifacts.
  2. **lint‑yaml** – Runs `yamllint` plus a custom schema validator (e.g., `yq` or `ajv`) on every `*.yaml` file under `apps/`, `roles/`, and any config files.
  3. **test** – Spins up a Docker‑in‑Docker service, executes end‑to‑end tests that cover the full install/stop flow.
  4. **docker‑build** – Builds the Docker image, tags it with the commit SHA and `latest`.
  5. **security‑scan** – Executes **Trivy** (or `grype`) on the built image; the job fails if any CVE severity ≥ 7 is found.
  6. **push** – Pushes the image to Docker Hub using secrets `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN`.
- **Trigger** – Runs on every push to `main` and on pull‑request events.
- **Caching** – Uses `actions/cache` for Cargo registry (`~/.cargo/registry`) and Docker layers to speed up subsequent runs.
- **Artifacts** – Test coverage reports are uploaded as build artifacts.
- **Fail‑fast** – Any step that returns a non‑zero exit code aborts the pipeline.

## Technical Constraints & Edge Cases

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

# User Story 20 – Docker Secrets for Sensitive Environment Variables

As an administrator, I want to store secret values (e.g., DB passwords, API keys) in Docker secrets and manage them from the Admin Panel.

## Acceptance Criteria

- **Secret declaration** – In `docker‑compose.yml` each container includes a `secrets:` entry, e.g.:

  ```yaml
  secrets:
    - source: db_password
      target: DB_PASSWORD
  ```

- **Admin Panel – Secrets page**:
  - **Create** – Admin provides a key/value pair; the backend calls `docker secret create <name> <value>` (or uses the Docker API).
  - **List** – Shows all existing secret names and their current values (masked).
  - **Update** – Overwrites the secret value (`docker secret rm` + `docker secret create`).
  - **Delete** – Removes the secret (`docker secret rm`); the backend also clears any reference in the composed YAML.
- **Runtime usage** – When the container is started via `docker compose up`, Docker automatically injects the secret as an environment variable (`DB_PASSWORD`) inside the container.
- **No secret exposure** – The backend never logs the secret value; logs only reference the secret name (if needed for audit).
- **Security** – Secrets are stored encrypted on the Docker host and are not baked into image layers.

## Technical Constraints & Edge Cases

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

# User Story 21 – Backup Configuration & Retention

As a product manager, I want the system to let the admin configure backup retention (how many days to keep backups and how many backup files to keep). By default a backup runs before an app is installed and a nightly backup is taken.

## Acceptance Criteria

- **Admin UI – Backup Settings**:
  - **Retention days** – integer ≥ 1 (default 30).
  - **Max backups per day** – integer ≥ 1 (default 7).
- **Pre‑install backup** – When the user clicks **Install**, a snapshot is taken of:
  - The app’s data volume (mounted volume).
  - The YAML file (including any user‑provided values) in `enabled/<slug>.yaml`.
  - The backup is stored under `/backups/<tenant>/` with a timestamped filename.
- **Nightly backup** – A cron‑style job (run at 02:00 UTC) creates a timestamped backup of:
  - All persisted volumes (app data).
  - The `enabled/` and `disabled/` YAML directories for the current tenant.
- **Retention policy** – Backup files are rotated:
  - Keep the newest **N** files per day (where **N** = _Max backups per day_).
  - Delete older files beyond the _Retention days_ window.
- **Logging** – Each backup operation logs: start time, success/failure, duration, unique backup ID, and involved resources.
- **Storage** – Backups are stored on a mounted volume or object storage (configurable via env var).

## Technical Constraints & Edge Cases

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

# User Story 22 – Rollback Mechanism During Installation

As a product manager, I need a rollback system that automatically reverts the container and YAML if the health‑check fails or any step after launch fails.

## Acceptance Criteria

- **Record‑keeping** – When the backend creates a container from a YAML, it stores:
  1. Container ID.
  2. Exact YAML version (hash or timestamp).
  3. Snapshot of the data volume (if any).
- **Health‑check monitoring** – The backend polls the container’s health status (Docker `health_status` or a readiness‑endpoint) for up to **2 minutes**.
- **Rollback trigger** – If health is not **healthy** within that window:
  - Backend executes `docker rm -f <container_id>`.
  - Restores the previous YAML version (if a backup exists) or re‑applies the last known good state.
  - Returns **409 Conflict** with a user‑friendly message: “Installation failed – automatic rollback performed.”
- **UI feedback** – The Angular UI shows a spinner with “Rollback in progress…”, then updates the app status to “Installation Failed – Rolled back” and displays the error toast.
- **Audit log** – Records: who initiated install, when rollback occurred, which container was removed, and which YAML version was restored.

## Technical Constraints & Edge Cases

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

# User Story 23 – Single‑Tenant Isolation (Multi‑Tenant = TODO)

As a product manager, I want the application to operate in a single‑tenant environment now, with multi‑tenant support marked as a TODO for a future release.

## Acceptance Criteria

- **Tenant ID** – Each request includes a tenant identifier (e.g., from the JWT claim `tenant_id` or a session variable).
- **Resource namespacing** – All Docker resources (networks, volumes, backup directories) are prefixed with the tenant ID (`tenant_<id>_backend`, `tenant_<id>_data`, `/backups/tenant_<id>/`).
- **UI** – The current tenant is shown in a dropdown (or hidden field) on the header; the backend enforces that a user can only act on apps belonging to that tenant.
- **Future‑proof** – The codebase is structured so that adding true multi‑tenant support (e.g., separate networks per tenant, isolated DB schemas) requires only minor changes.

## Technical Constraints & Edge Cases

**Technical Constraints**

- Every request includes a **tenant identifier** (from JWT claim `tenant_id` or session variable).
- Docker resources (networks, volumes, backup directories) are **prefixed** with the tenant ID (`tenant_<id>_backend`, `tenant_<id>_data`, `/backups/tenant_<id>/`).
- UI shows current tenant in a dropdown (or hidden field); backend enforces that a user can act only on apps belonging to that tenant.
- Architecture is **future‑proof** for multi‑tenant: code uses tenant‑scoped names, no hard‑coded global names; adding true multi‑tenant (separate DB schemas, per‑tenant isolation) would require minimal changes.

**Edge Cases**

- Tenant ID missing or invalid → 400 Bad Request.
- Two tenants using the same Docker network name (e.g., `tenant_1_net`) → conflict; system must enforce uniqueness via naming scheme.
- Tenant‑specific resource leak (e.g., orphaned volume) → periodic cleanup job recommended.

# User Story 24 – SSL Certificate Management (Let’s Encrypt & Custom)

As a product manager, I want the application to use Let’s Encrypt for automatic certificate provisioning, with an option for admins to upload their own certificates.

## Acceptance Criteria

- **Admin UI – SSL Settings**:
  - **Auto‑SSL toggle** – default **ON**.
  - **Custom certificate upload** – fields for PEM‑encoded certificate and private key.
- **Auto‑SSL flow**:
  - For each newly installed app, the backend requests a Let’s Encrypt certificate for `app‑<slug>.example.com` using an ACME client (e.g., **cert‑manager** or **acme.sh**).
  - The obtained cert/key are stored as a Docker secret (`tls‑cert`) and referenced in the Traefik router configuration.
- **Custom cert flow**:
  - Admin uploads PEM files → backend creates a Docker secret (`tls‑custom`).
  - Traefik router is updated to use this secret for HTTPS termination on the app’s sub‑domain.
- **Certificate status** – UI shows current status (valid, expiring within X days, error) and a **Renew** button that forces a Let’s Encrypt renewal.
- **Secret handling** – Secrets are mounted read‑only into the Traefik container; the backend never logs the certificate contents.
- **Fallback** – If Let’s Encrypt fails (rate‑limit, DNS challenge error), the UI displays a clear message and logs the failure.

## Technical Constraints & Edge Cases

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

# User Story 25 – Real‑Time Install Status & Rate Limiting

As an end‑user, I want to see a live “Installing…” indicator for an app and be prevented from spamming the Install button.

## Acceptance Criteria

- **Rate limiting**:
  - **Global** – max **1** install request per user per **30 seconds**.
  - **Per‑app** – max **1** install request per app per **5 seconds**.
  - Excess requests receive **429 Too Many Requests** with a friendly message.
- **Install flow**:
  - Clicking **Install** sets the app’s status to **“Installing…”** in the **My Apps** list.
  - Backend starts the container(s) and begins health‑check polling.
- **Live status**:
  - Backend pushes status updates via **Server‑Sent Events (SSE)** or **WebSocket** to all connected clients, so every user sees the same progress in real time.
  - The UI shows a spinner and “Installing…” text while the container is starting.
- **Completion**:
  - On success, status changes to **“Active”**, the sub‑domain URL is displayed, and the app appears in **My Apps** with status **Active**.
  - On failure, status changes to **“Failed”** with an error toast/message.
- **Audit** – Each install attempt is logged (user ID, app slug, start time, outcome).

## Technical Constraints & Edge Cases

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

# User Story 26 – CORS & CSRF Protection

As a security analyst, I want all API calls to be protected against cross‑origin requests and CSRF attacks.

## Acceptance Criteria

- **CORS** – The backend allows only the origin(s) of the Angular SPA (e.g., `https://app.example.com`). All other origins receive **403 Forbidden**.
- **CSRF token**:
  - The SPA embeds a CSRF token (e.g., as a meta tag `<meta name="csrf-token" content="…">` or via an HttpOnly cookie).
  - For state‑changing POST/PUT/DELETE endpoints, the backend verifies the token before processing.
- **Security headers** – All responses include the following headers (see Story 32):
  - `Content‑Security‑Policy` (nonce‑based for inline scripts).
  - `X‑Content‑Type‑Options: nosniff`
  - `X‑Frame‑Options: DENY`
  - `Referrer‑Policy: strict-origin-when-cross-origin`
  - `Strict‑Transport‑Security` (when HTTPS is active).
  - `X‑XSS‑Protection: 1; mode=block` (legacy browsers).
- **Enforcement** – Headers are added both by the reverse‑proxy (Traefik) and by the backend middleware, and cannot be disabled via the admin UI.

## Technical Constraints & Edge Cases

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

# User Story 27 – Graceful Error Handling & User‑Friendly Messages

As a product manager, I want all errors returned to the UI to be translated into clear, actionable messages.

## Acceptance Criteria

- **Error format** – Every error response contains:
  1. `code` (e.g., `ERR_IMAGE_NOT_FOUND`).
  2. `message` (concise, non‑technical).
  3. Optional `details` (e.g., “Please verify the image name and try again”).
- **User‑friendly** – Messages avoid jargon and suggest next steps (e.g., “Image not found – verify the image name”, “Insufficient memory – reduce the requested RAM”).
- **Logging** – Each error is logged server‑side with timestamp, user ID, endpoint, and a hash of the payload (no secrets).
- **UI integration** – Angular error interceptor catches HTTP errors, maps them to the appropriate toast/alert component, and never shows raw stack traces to the user.

## Technical Constraints & Edge Cases

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

# User Story 28 – Per‑App Docker Network & Ingress

As a product manager, I want each app to run in its own Docker network while still allowing the configuration app to reach it.

## Acceptance Criteria

- **Network creation** – When an app is launched, the backend creates a dedicated Docker network named `app_<slug>` (e.g., `app_myapp`).
- **Container attachment** – The app’s container is connected to its network; the configuration (installer) container is attached to a special **admin** network that spans all app networks it needs to communicate with.
- **Ingress** – Traefik is configured with a router per app that forwards traffic on the app’s sub‑domain (`app‑<slug>.example.com`) to the correct container network.
- **Isolation** – Containers cannot directly communicate with other apps’ networks unless explicitly allowed by a network‑policy rule (out of scope for now).

## Technical Constraints & Edge Cases

**Technical Constraints**

- When an app is launched, backend creates a dedicated Docker network named `app_<slug>` (e.g., `app_myapp`).
- The app’s container is attached to its network; the configuration/installer container is attached to an **admin** network that spans all app networks it needs to communicate with.
- Traefik router per app forwards traffic on the app’s sub‑domain (`app‑<slug>.example.com`) to the correct container network.
- Containers cannot directly communicate with other apps’ networks unless an explicit network‑policy rule is added (out of scope).

**Edge Cases**

- Network name collision (two apps with same slug) → backend must ensure uniqueness (e.g., append tenant ID).
- Traefik router misconfiguration (wrong host rule) → traffic may not reach the app; UI may show “App not reachable”.
- Network creation fails (e.g., driver not available) → 500 error; UI shows “Network creation failed”.

# User Story 29 – Log Retention & Daily Plain‑Text Logs

As an admin, I want to define a log retention period and daily log size limit, with logs stored as one plain‑text file per day.

## Acceptance Criteria

- **Admin UI – Log Settings**:
  - **Log retention (days)** – integer ≥ 1 (default 90).
  - **Max log size per day** – integer (bytes) (default 10 MB).
- **Daily rotation** – At midnight the backend renames the active log to `app‑<slug>-YYYY‑MM‑DD.log`.
- **Size enforcement** – If a log file exceeds the max size, a new file is started for the same day (or the current file is truncated, as configured).
- **Storage** – Logs are stored as plain text (no compression) in `/logs/<tenant>/` (or a configurable path).
- **API** – `GET /logs?appSlug=…&date=YYYY-MM-DD` streams the requested day’s log.
- **Log entry format** – Each line includes: timestamp, log level, component, and optionally user ID.

## Technical Constraints & Edge Cases

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

# User Story 30 – API Versioning

As a product manager, I want all public APIs to be versioned and the version to appear in the URL path.

## Acceptance Criteria

- **Version prefix** – All endpoints are prefixed with `/api/v1` (future versions become `/api/v2`, etc.).
- **OpenAPI versioning** – The Swagger document lives at `/docs/v1` (UI) and `/docs-json/v1` (raw spec). When a new version is released, the path and spec version are updated accordingly.
- **Backward compatibility** – Non‑breaking changes are introduced in a new version; old versions remain supported for at least **12 months**.
- **Deprecation notices** – Any endpoint slated for removal is marked as **deprecated** in the Swagger UI and a comment in the spec.

## Technical Constraints & Edge Cases

**Technical Constraints**

- All public APIs prefixed with `/api/v1` (future versions become `/api/v2`, etc.).
- Swagger spec lives at `/docs/v1` (UI) and `/docs-json/v1` (raw). Version number in spec matches URL.
- Backward‑compatible changes introduced in a new version; old versions stay supported for at least **12 months**.
- Deprecated endpoints are marked in Swagger UI and annotated in the spec (e.g., `deprecated: true`).

**Edge Cases**

- Old client still calling `/api/v1` after a major version bump → may break if non‑backward‑compatible changes were made; versioning must preserve compatibility.
- Documentation drift (spec not updated after code change) → CI validation will fail; ensure CI runs spec validation on every PR.

# User Story 31 – Accessibility & Internationalization (i18n)

As a product manager, I want the UI to be WCAG‑2.1 AA compliant and support English and Dutch.

## Acceptance Criteria

- **WCAG compliance** – All interactive elements meet contrast ratios, have proper ARIA labels, maintain a logical focus order, and are fully operable with keyboard only.
- **i18n implementation** – Angular uses **ngx‑translate** (or similar) with separate JSON files:
  - `en.json` – English strings.
  - `nl.json` – Dutch strings.
- **Externalisation** – Every visible UI string (buttons, error messages, placeholders, tooltips) is sourced from the translation files.
- **Language selector** – A dropdown (or flag icon) in the top‑right corner lets the user choose the language; the choice is persisted via a cookie or `localStorage`.
- **Fallback** – If a translation key is missing, the UI falls back to the English string and logs a warning for the developer.

## Technical Constraints & Edge Cases

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

# User Story 32 – Security Headers & Hardening

As a security analyst, I want security headers (CSP, X‑Frame‑Options, HSTS, Referrer‑Policy, etc.) applied globally to every response.

## Acceptance Criteria

- **Middleware** – A Rust (or Traefik) middleware adds the following headers to **every** response:
  1. `Content‑Security‑Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';` (adjust as needed for nonces).
  2. `X‑Content‑Type‑Options: nosniff`
  3. `X‑Frame‑Options: DENY`
  4. `Referrer‑Policy: strict-origin-when-cross-origin`
  5. `Strict-Transport-Security: max-age=31536000; includeSubDomains; preload` (only when HTTPS is active).
  6. `X‑XSS‑Protection: 1; mode=block` (legacy browsers).
- **Enforcement** – Headers are set both at the edge (Traefik) and by the backend; they cannot be turned off via the admin UI.
- **CSP nonce** – Inline scripts/styles use a cryptographic nonce generated per request, keeping the policy tight while allowing necessary inline code.
- **Testing** – Automated tests verify that the headers are present on all routes (including static assets).

## Technical Constraints & Edge Cases

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
