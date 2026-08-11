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
- **CORS** – Only the Angular SPA origin (`https://app.local`) is allowed; all other origins receive **403 Forbidden**.

## Technical Constraints & Edge Cases

**Technical Constraints**

- Angular UI components that trigger container actions must be **conditionally rendered** only when the JWT contains `admin` role (Angular guard).
- Backend must **re‑check** the JWT role on every container‑management request; non‑admin → 403 Forbidden.
- Secrets (Docker secrets) must never be included in any API response; only high‑level status fields are returned.
- Audit logging for each admin action: record user ID, endpoint, outcome (success/failure), container ID (if applicable).
- CORS policy: only `https://app.local` allowed; any other origin → 403 Forbidden.

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

- **CORS** – The backend allows only the origin(s) of the Angular SPA (e.g., `https://app.local`). All other origins receive **403 Forbidden**.
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

- **CORS**: backend allows only origin `https://app.local other origins receive **403 Forbidden**.
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

---

# Behavioral Nudge Engine User Stories (US-33 – US-40)

> Derived from `docs/NUDGE_ENGINE.md`. The Behavioral Nudge Engine (BNE) is a
> lightweight, policy-driven subsystem that steers administrators and end-users
> toward safer, healthier, and more efficient operations without removing their
> freedom of choice. Nudge IDs use the `N-<domain>-<n>` convention for
> traceability into the audit log.

# User Story 33 – Nudge Engine Core Infrastructure

As a platform engineer, I want a backend nudge policy module, a persistent nudge store, an SSE delivery channel, and a frontend `<sam-nudge>` component, so that nudges can be evaluated, stored, delivered, and rendered consistently across the platform.

## Acceptance Criteria

- **Backend policy module** – A `NudgePolicy` module in the Rust/axum backend subscribes to the existing domain event bus (install success, health failure, secret age tick, etc.) and evaluates events against configurable rules.
- **Rule evaluation** – For each event, `NudgePolicy` checks `NudgeStore` for snooze/opt-out/already-seen state, selects a variant, and emits a nudge DTO.
- **NudgeStore (Postgres)** – A `nudges` table persists one row per nudge instance with columns: `id` (UUID), `user_id`, `tenant_id`, `nudge_id` (e.g. `N-SEC-1`), `variant`, `trigger_event` (JSONB), `state` (`pending|shown|acted|snoozed|dismissed`), `shown_at`, `acted_at`, `snoozed_until`, `created_at`.
- **Preferences table** – A `nudge_prefs` table stores per-user opt-out state: `user_id` (PK), `reduce_suggestions` (boolean, default false), `safety_snooze_max_hours` (int, default 72).
- **SSE delivery** – Nudge DTOs are pushed over the same SSE channel used for live install status (FR-RT-1); a REST fallback (`GET /api/v1/nudges`) returns active nudges on page load.
- **Frontend NudgeService** – An Angular `NudgeService` consumes the SSE stream with REST fallback, handles deduplication, snooze, and opt-out state.
- **`<sam-nudge>` component** – Renders in one of three slots: dashboard banner, inline next to the related entity, or transient toast. Uses design-system tokens from `styles.css` (no new visual language).
- **API surface** – Additive to the existing OpenAPI spec:
  | Method | Path | Role | Purpose |
  |--------|------|------|---------|
  | `GET` | `/api/v1/nudges` | any auth | active nudges for the current user |
  | `POST` | `/api/v1/nudges/{id}/act` | any auth | record an action outcome |
  | `POST` | `/api/v1/nudges/{id}/snooze` | any auth | snooze with bounded duration |
  | `POST` | `/api/v1/nudges/{id}/dismiss` | any auth | dismiss (logged) |
  | `GET`/`PATCH` | `/api/v1/nudges/prefs` | any auth | read/update opt-out prefs |
- **Audit integration** – All nudge lifecycle events (trigger, shown, acted, snoozed, dismissed) are written to the same audit log used for admin actions (FR-ADMIN-3).
- **Tenant scoping** – All endpoints are JWT + tenant-scoped; a user only ever sees their own nudges.

## Technical Constraints & Edge Cases

**Technical Constraints**

- The BNE is **not** a separate service; it is a backend policy module plus a frontend directive that reuses existing SAM infrastructure.
- `NudgePolicy` rules must be data-driven (editable without redeploy) so that triggers, audiences, and variants can be tuned operationally.
- The `nudges` and `nudge_prefs` tables must be created via sqlx migrations alongside the existing schema.
- The SSE channel must be the same one used for FR-RT-1 (live install status); nudges are low-frequency and must not saturate it.
- `<sam-nudge>` must render exclusively with design-system tokens (`--warn`, `--bad`, `--info`, `--ok`, `--accent`); no new CSS classes outside `styles.css`.
- All five API endpoints require a valid JWT (HttpOnly cookie) and are audited like other admin actions.
- The nudge lifecycle is: Trigger → Evaluate → Deliver → Render → Resolve → Measure.

**Edge Cases**

- SSE connection drops mid-stream → `NudgeService` falls back to REST `GET /api/v1/nudges` on reconnect; no nudge is lost.
- A nudge fires for a user who has opted out of non-safety nudges → `NudgePolicy` suppresses it; safety-critical nudges (N-SEC-3, N-REL-1) bypass the opt-out but respect snooze.
- Duplicate trigger events arrive in rapid succession → `NudgeStore` deduplicates by `(user_id, nudge_id, trigger_event)` within a cooldown window.
- `NudgeStore` is unavailable (Postgres down) → nudge evaluation degrades gracefully; no nudge is delivered but domain operations continue unaffected.
- A nudge references a page or entity that no longer exists (e.g., app was uninstalled) → the nudge is auto-dismissed on next evaluation.

# User Story 34 – Nudge Transparency, Opt-out & Ethical Guardrails

As an end-user or administrator, I want every nudge to be transparent, dismissible, and opt-out-able, with safety-critical nudges remaining snoozable but not permanently hideable, so that I trust the platform is steering me ethically rather than manipulating me.

## Acceptance Criteria

- **"Why am I seeing this?"** – Every rendered `<sam-nudge>` includes a "Why am I seeing this?" affordance that opens a panel explaining the nudge's trigger, the behavioral lever applied, and the policy rule ID.
- **Opt-out preference** – A per-user "Reduce suggestions" toggle (`nudge_prefs.reduce_suggestions`) disables all non-safety nudges. The toggle is accessible from the user profile or settings page.
- **Safety snooze cap** – Safety-critical nudges (N-SEC-3, N-REL-1) cannot be permanently dismissed; they can be snoozed for a bounded period up to `safety_snooze_max_hours` (default 72h). After the snooze expires, the nudge re-evaluates.
- **No dark patterns** – Nudges never hide cheaper or safer options, never use fake urgency (e.g., "only 1 left!"), and never block a legitimate action the user is authorized to perform.
- **Proportionality** – Nudge intensity (color, placement, persistence) scales with operational risk, not with engagement metrics. A misconfigured nudge cannot become a nag.
- **Per-user rate cap** – A maximum of N active nudges are shown concurrently per user (configurable, default 3) to prevent nudge fatigue and banner blindness.
- **Audit trail** – Every nudge shown, acted upon, snoozed, or dismissed is logged with timestamp, user ID, nudge ID, variant, and outcome.
- **Dismiss + snooze controls** – Each `<sam-nudge>` renders a dismiss (×) and snooze (clock) control alongside the "Why am I seeing this?" link.

## Technical Constraints & Edge Cases

**Technical Constraints**

- The "Why am I seeing this?" panel must reference the nudge's `nudge_id`, the trigger event summary, and a human-readable description of the behavioral lever (from §2.2 of `NUDGE_ENGINE.md`).
- The `reduce_suggestions` preference must be respected by `NudgePolicy` at evaluation time, not at render time (suppressed nudges are never created).
- Safety nudges are identified by a `safety: true` flag in the policy rule definition; the snooze cap is enforced server-side via `snoozed_until` validation.
- The per-user active-nudge cap is enforced by `NudgePolicy` before delivery; excess nudges remain in `pending` state until a slot opens.
- All four ethical guardrails (transparency, opt-out, no dark patterns, proportionality) must be verifiable via the audit log.

**Edge Cases**

- User toggles "Reduce suggestions" while a non-safety nudge is already shown → the nudge is dismissed immediately and the preference is applied to future evaluations.
- User attempts to snooze a safety nudge beyond `safety_snooze_max_hours` → the snooze request is clamped to the maximum and the UI informs the user.
- User dismisses a nudge but the underlying condition persists → the nudge re-evaluates on the next trigger event (dismissal is per-instance, not per-condition).
- Rate cap is reached and a new safety nudge fires → safety nudges bypass the rate cap; only non-safety nudges are queued.
- Admin disables a nudge rule via policy edit → all `pending` instances of that nudge are auto-dismissed; `shown` instances remain until resolved.

# User Story 35 – Security Domain Nudges

As an administrator, I want security-focused nudges that remind me to rotate stale secrets, default to least-privilege role assignments, and warn me about backup gaps with loss-framed messaging, so that I close security gaps caused by inattention and status quo bias.

## Acceptance Criteria

- **N-SEC-1 · Secret rotation reminder**:
  - **Trigger** – A Docker secret's `last_rotated_at` exceeds the policy threshold (default 90 days) or is null.
  - **Audience** – `admin` users on `admin-secrets.html`.
  - **Mechanism** – A `--warn` banner appears above the secrets table listing expiring keys with a "Rotate selected" button that pre-fills the rotation modal.
  - **Lever** – Salience + timely + friction.
- **N-SEC-2 · Strong-defaults on new role assignment**:
  - **Trigger** – Admin opens the "Add user" or "Edit role" dialog on `admin-users.html`.
  - **Mechanism** – The role `<select>` defaults to `user` (least privilege). Helper text shows the permission delta vs. `admin` in plain language (e.g., "Grants read:apps, write:containers — no user or settings management").
  - **Lever** – Defaults + framing.
- **N-SEC-3 · Loss-framed backup gap warning**:
  - **Trigger** – No successful backup in `retention_interval * 1.5` OR a backup failure is logged.
  - **Audience** – `admin` on the dashboard (`home.html`) and `admin-backups.html`.
  - **Mechanism** – The dashboard "Last Backup" stat tile flips from `--ok` to `--warn`/`--bad` and the delta line reads "No backup in 6h — 12 apps at risk" instead of the neutral timestamp. A "Back up now" button is rendered inline.
  - **Lever** – Framing (loss) + salience + timely + friction.
  - **Safety** – This is a safety-critical nudge: opt-out does not suppress it; snooze is capped at `safety_snooze_max_hours`.
- **Measurement** – Each nudge records its designated metric:
  - N-SEC-1: `secret_age_days` p50/p90; rotation events per admin per 30 days.
  - N-SEC-2: share of new users created with `admin` role; revert events within 24h.
  - N-SEC-3: `hours_since_last_good_backup` p90; manual backup trigger rate within 1h of warning.

## Technical Constraints & Edge Cases

**Technical Constraints**

- N-SEC-1 requires the `secrets` table to track `last_rotated_at`; the policy threshold is configurable (default 90 days).
- N-SEC-2 requires the role `<select>` in the "Add user"/"Edit role" dialog to default to `user`; the permission delta text is derived from `roles/roles.yaml`.
- N-SEC-3 requires the dashboard "Last Backup" `.stat` tile to support dynamic color (`--ok`/`--warn`/`--bad`) and a dynamic delta line with an inline action button.
- N-SEC-3 is flagged `safety: true` in the policy definition; it bypasses `reduce_suggestions` and enforces the snooze cap.
- All three nudges are delivered via the SSE channel and rendered through `<sam-nudge>` using existing design-system tokens.

**Edge Cases**

- All secrets are newly created (`last_rotated_at` is null) → N-SEC-1 lists all secrets as expiring; the banner should summarize count rather than list every key if > 10.
- Admin changes a role from `admin` to `user` (downgrade) → N-SEC-2 does not fire (the nudge is for new assignments and upgrades, not downgrades).
- Backup is running at the moment the trigger evaluates → N-SEC-3 does not fire if a backup is in progress (state = `running`); it fires only on confirmed gap or failure.
- No apps are installed → N-SEC-3 delta line reads "No backup in 6h" without the "apps at risk" clause (zero apps).
- The rotation modal is already open when N-SEC-1 fires → the banner is suppressed to avoid duplicate UI.

# User Story 36 – Reliability Domain Nudges

As an administrator or end-user, I want reliability-focused nudges that help me triage unhealthy containers quickly, confirm destructive uninstalls with dependency context, and set resource limits on launched containers, so that I reduce mean time to remediation and avoid accidental data loss.

## Acceptance Criteria

- **N-REL-1 · Unhealthy container triage nudge**:
  - **Trigger** – A container's health check fails (FR-CL-2 SSE event).
  - **Audience** – Any user with `read:containers`; admins get an additional action.
  - **Mechanism** – The dashboard "Unhealthy" stat tile links directly to a filtered `containers.html?health=unhealthy` view. For admins, a "Restart + recheck" button appears next to the row without leaving the dashboard.
  - **Lever** – Salience + friction + timely.
  - **Safety** – Safety-critical nudge; bypasses opt-out, snooze capped.
  - **Measurement** – `mttr_unhealthy_minutes` p50/p90; restarts issued from dashboard vs. containers page.
- **N-REL-2 · Dependency-aware uninstall confirmation**:
  - **Trigger** – User clicks Uninstall on an app with running dependents (FR-UNI-2).
  - **Mechanism** – The confirmation modal lists dependent containers with their health state and requires a typed confirmation of the app slug (not a generic "Are you sure?"). The dependent list is the salient element; the type-to-confirm is the friction.
  - **Lever** – Salience + friction + framing (loss of dependents).
  - **Measurement** – Uninstalls aborted at confirmation; dependents left orphaned (target: 0).
- **N-REL-3 · Resource-limit suggestion on launch**:
  - **Trigger** – Admin launches a child container (FR-CL-1) without `mem_limit`/`cpu_quota`.
  - **Mechanism** – The launch form shows a non-blocking `--info` hint with the host's current free memory and a suggested limit band. The field is pre-filled with the suggestion but editable.
  - **Lever** – Defaults + social proof (host state) + friction.
  - **Measurement** – Share of launched containers with explicit limits; OOM events per 100 container-hours.

## Technical Constraints & Edge Cases

**Technical Constraints**

- N-REL-1 requires the dashboard "Unhealthy" `.stat` tile to be a deep link to `containers.html?health=unhealthy`; the containers page must support URL-filtered health state on load.
- N-REL-1 admin "Restart + recheck" action requires `write:containers` permission; the action calls the existing container restart endpoint and triggers an immediate health re-check.
- N-REL-2 requires the uninstall modal to query running dependents before rendering; the typed confirmation must match the app slug exactly (case-sensitive).
- N-REL-3 requires the launch form to query host free memory (via Docker SDK `docker system df` or `inspect`) and pre-fill `mem_limit`/`cpu_quota` fields with a suggested band.
- N-REL-1 is flagged `safety: true`; N-REL-2 and N-REL-3 are not safety-critical and respect `reduce_suggestions`.

**Edge Cases**

- Multiple containers go unhealthy simultaneously → N-REL-1 fires once with a summary ("3 containers unhealthy"); the deep link filters to all unhealthy.
- Admin clicks "Restart + recheck" but the container is already restarting → the action is idempotent; no duplicate restart is issued.
- App has no running dependents → N-REL-2 does not fire; the standard uninstall confirmation (US-9) is shown.
- User types the app slug with trailing whitespace → confirmation is trimmed before comparison.
- Host has abundant free memory → N-REL-3 suggestion is conservative (not max); the hint clarifies "suggested baseline, adjust as needed."
- Admin manually overrides the pre-filled limit to a very high value → N-REL-3 does not warn again (the nudge is a suggestion, not a validation).

# User Story 37 – Adoption Domain Nudges

As an end-user or administrator, I want adoption-focused nudges that encourage me to configure backups after install, surface popular apps via social proof, and notify me when app definitions are stale, so that I get to value faster and keep my apps up to date.

## Acceptance Criteria

- **N-ADOPT-1 · Post-install backup commitment**:
  - **Trigger** – A successful app install (FR-APP-1 success event).
  - **Mechanism** – The post-install summary card (FR-APP-3) gains an optional checkbox: "Remind me to configure backups for {{app}} in 24h." If checked, a single timed nudge fires in 24h; if ignored, no nag.
  - **Lever** – Commitment + timely.
  - **Measurement** – % of installed apps with a backup within 7 days of install.
- **N-ADOPT-2 · App store social-proof refinement**:
  - **Trigger** – Rendering of an `.app-card` on `apps.html`.
  - **Mechanism** – The existing install count is augmented with a contextual line for apps the user's tenant hasn't installed: "Popular in Media — installed by 8 of 10 tenants." In single-tenant MVP, fall back to global install count + "Recently added" badge.
  - **Lever** – Social proof.
  - **Measurement** – Install-through-rate per card; 24h uninstall rate (guardrail — must not increase).
- **N-ADOPT-3 · Stale app definition update nudge**:
  - **Trigger** – An enabled app's YAML `version` is older than the store version.
  - **Audience** – `admin` on `admin-apps.html`.
  - **Mechanism** – A row-level `--info` badge "Update available" with a one-click "Diff & upgrade" action that opens the YAML editor pre-loaded with the new version and a highlighted diff.
  - **Lever** – Salience + friction + timely.
  - **Measurement** – `app_definition_age_days` p50; upgrade actions per week.

## Technical Constraints & Edge Cases

**Technical Constraints**

- N-ADOPT-1 requires the post-install summary card to render an optional checkbox; if checked, `NudgeStore` creates a `pending` nudge with `snoozed_until = now + 24h`. If unchecked, no nudge is created.
- N-ADOPT-1 fires at most once per install; if the user ignores the 24h nudge, no follow-up nag is sent.
- N-ADOPT-2 requires the `.app-card` footer to render a contextual social-proof line; in single-tenant MVP, the source is global install counts (not per-tenant).
- N-ADOPT-2 must not increase the 24h uninstall rate (guardrail from §6.2 of `NUDGE_ENGINE.md`).
- N-ADOPT-3 requires the `admin-apps.html` table to compare enabled app YAML versions against store versions; the "Diff & upgrade" action opens the YAML editor with a highlighted diff view.
- All three nudges are non-safety and respect `reduce_suggestions`.

**Edge Cases**

- User installs an app and checks the backup commitment box, then configures backups before 24h → the timed nudge is auto-dismissed when backup configuration is detected.
- User installs an app and does not check the box → no nudge is ever sent (no nag).
- Single-tenant deployment with no install history → N-ADOPT-2 falls back to "Recently added" badge only; no fabricated social-proof numbers.
- App YAML version matches store version → N-ADOPT-3 does not fire; no badge is shown.
- "Diff & upgrade" is clicked but the new YAML version has breaking schema changes → the editor surfaces a validation warning before save (US-18 schema validation).
- Multiple apps have updates available → N-ADOPT-3 fires per-row (inline badge), not as a single banner; the rate cap applies to banner-slot nudges only.

# User Story 38 – Hygiene Domain Nudges

As a new tenant or an administrator, I want hygiene-focused nudges that guide me through first-time setup with suggested starter apps and highlight anomalous audit events, so that I reach first value quickly and review high-risk actions promptly.

## Acceptance Criteria

- **N-HYG-1 · Empty-state onboarding nudge**:
  - **Trigger** – A fresh tenant with zero installed apps opens the dashboard.
  - **Mechanism** – The "My Apps" preview is replaced with a guided empty state: three suggested starter apps (e.g., a media app, a productivity app, a dev tool) with one-click install and a "Set up nightly backups" secondary CTA.
  - **Lever** – Defaults + social proof + friction.
  - **Measurement** – Time-to-first-install; time-to-first-backup-config.
- **N-HYG-2 · Audit-log anomaly highlight**:
  - **Trigger** – The recent-activity feed contains a high-risk event (role change, secret rotation, app deletion, failed login burst).
  - **Mechanism** – The dashboard activity row renders with a `--warn`/`--bad` left border and a "Review" link to the filtered audit view.
  - **Lever** – Salience + timely.
  - **Measurement** – Time-from-event-to-review for flagged vs. unflagged events.

## Technical Constraints & Edge Cases

**Technical Constraints**

- N-HYG-1 requires the dashboard to detect a zero-app state (query installed app count on load) and render the guided empty state instead of the "My Apps" table preview.
- N-HYG-1 suggested starter apps are sourced from the app store catalog with a `starter: true` or `featured: true` flag in the YAML; fallback is the three most-installed apps.
- N-HYG-1 "Set up nightly backups" CTA links to `admin-settings.html` (or `admin-backups.html`) with a pre-focused backup configuration section.
- N-HYG-2 requires the recent-activity feed to classify events by risk level; high-risk events are defined as: role change, secret rotation, app deletion, failed login burst (> 5 in 5 min).
- N-HYG-2 "Review" link deep-links to `admin-backups.html` (audit log tab) filtered by the flagged event type.
- Both nudges are non-safety and respect `reduce_suggestions`.

**Edge Cases**

- Tenant installs an app then uninstalls it (back to zero) → N-HYG-1 does not re-fire (it is a first-run nudge only; detected via a `first_install_completed` flag in tenant state).
- No apps in the catalog have `starter: true` → N-HYG-1 falls back to the three most-installed apps globally; if no install history exists, shows three alphabetically first apps.
- Multiple high-risk events occur in the same activity window → N-HYG-2 highlights each row independently; the rate cap does not apply to inline row highlights.
- A flagged event is already reviewed (admin clicked through) → the left border is removed on next dashboard load; the "Review" link is replaced with "Reviewed" text.
- Failed login burst subsides before the admin sees the dashboard → the anomaly remains highlighted until explicitly reviewed or 24h elapses.

# User Story 39 – Nudge Measurement & Do-No-Harm Guardrails

As a platform engineer or product owner, I want per-nudge metrics and automated guardrails that detect when a nudge is causing harm (regret installs, dismissal fatigue, low action rates, audit flooding), so that I can tune or disable underperforming nudges without redeploying.

## Acceptance Criteria

- **Per-nudge metrics** – An offline job rolls up the following metrics per nudge ID, per tenant, on a daily basis:
  | Domain | Metric | Source |
  |--------|--------|--------|
  | Security | `secret_age_days` p50/p90 | `secrets` table |
  | Security | New users granted `admin` (%) | audit log |
  | Reliability | `mttr_unhealthy_minutes` p50/p90 | health events |
  | Reliability | Orphaned dependents post-uninstall | `containers` scan |
  | Reliability | OOM events / 100 container-hrs | Docker events |
  | Adoption | Time-to-first-install (new tenant) | install events |
  | Adoption | Time-to-first-backup-config | backup settings |
  | Hygiene | Time-from-event-to-review (flagged) | audit view events |
- **Guardrails (do no harm)**:
  1. **24h uninstall rate** must not rise with N-ADOPT-2 (social proof must not push regret installs).
  2. **Dismissal rate** per nudge > 60% over 30 days triggers a policy review (the nudge is likely noise or mistimed).
  3. **Action rate** per safety nudge < 20% triggers a salience/friction redesign, not a stronger block.
  4. **Audit log volume** from nudge events is kept < 5% of total to avoid drowning signal.
- **Alerting** – When a guardrail threshold is breached, a `--warn` alert is surfaced to admins on the dashboard and logged; the nudge rule is not auto-disabled but is flagged for review.
- **Observability** – Metrics are queryable via an admin endpoint (`GET /api/v1/nudges/metrics`) and visualized on a dedicated metrics view within `admin-settings.html` or `admin-backups.html`.

## Technical Constraints & Edge Cases

**Technical Constraints**

- The offline metrics job runs daily (scheduled task or cron) and writes rollups to a `nudge_metrics` table (or similar) keyed by `(nudge_id, tenant_id, date, metric_name, percentile)`.
- Guardrail thresholds are configurable in the nudge policy (not hardcoded) so they can be tuned without redeploy.
- Evaluation is observational first (before/after on the same tenant); A/B is deferred to US-40 (Phase 3).
- The audit log volume guardrail is measured as a rolling 7-day percentage; nudge events must be tagged with a `source: 'bne'` field for filtering.
- The metrics endpoint requires `admin` role.

**Edge Cases**

- A nudge is newly deployed and has < 30 days of data → guardrails do not evaluate until the minimum sample window is met (30 days or 100 instances, whichever comes first).
- A guardrail breach is caused by a one-time anomaly (e.g., a mass secret rotation event) → the alert is surfaced but auto-resolves if the next 7-day window returns to normal.
- The metrics job fails to run → a `--bad` alert is surfaced to admins; metrics staleness is visible on the metrics view.
- Audit log volume temporarily spikes due to a burst of nudge events (e.g., multiple unhealthy containers) → the 7-day rolling average smooths the spike; a single day over 5% does not trigger a breach.
- A nudge is disabled mid-evaluation-window → partial-window data is retained but guardrails are not evaluated for disabled nudges.

# User Story 40 – Nudge A/B Experimentation (Phase 3, Future)

As a product owner, I want to run A/B experiments on nudge variants (copy, placement, lever) gated on the do-no-harm guardrails, so that I can iteratively improve nudge effectiveness where action rates are marginal.

## Acceptance Criteria

- **Variant assignment** – `NudgePolicy` supports multiple variants per nudge ID (labelled A, B, C, …) with configurable traffic allocation percentages per tenant.
- **Sticky assignment** – A user assigned to a variant stays in that variant for the experiment duration (stored in `nudge_prefs` or a dedicated `nudge_experiments` table).
- **Guardrail gating** – An experiment cannot be started if any guardrail (US-39) is currently breached for the target nudge. If a guardrail breaches mid-experiment, the experiment auto-pauses and reverts to the control variant.
- **Metrics per variant** – The metrics job (US-39) rolls up per `(nudge_id, variant)` so action rates, dismissal rates, and guardrail metrics are comparable across variants.
- **Experiment lifecycle** – An experiment has states: `draft`, `running`, `paused`, `completed`, `reverted`. Only `admin` users can create, start, pause, or stop experiments.
- **Admin UI** – An "Experiments" tab on `admin-settings.html` lists active and past experiments with variant allocation, action rate, and guardrail status.

## Technical Constraints & Edge Cases

**Technical Constraints**

- This user story is **P3 / Future** (Phase 3 per `NUDGE_ENGINE.md` §7); it is gated on the guardrails and traffic from US-39 (Phase 0–2).
- Variant assignment must be deterministic per user (hash of `user_id + experiment_id`) to ensure sticky assignment without a lookup on every evaluation.
- The `nudges` table already has a `variant` column (US-33); experiments populate it with the assigned variant label.
- Guardrail auto-pause must revert all users to the control variant within one evaluation cycle; no user should see a breached variant after pause.
- Experiment configuration is data-driven (policy rules, not code paths); an experiment can be created, started, and stopped without redeploy.

**Edge Cases**

- Experiment is started with 50/50 allocation but only 5 users exist → results are not statistically significant; the UI should warn when sample size is below a minimum threshold.
- A user is assigned to variant B but the experiment is reverted to control → the user's next nudge evaluation uses the control variant; the `variant` column retains "B" for historical rows.
- Two experiments target the same nudge ID simultaneously → the policy module rejects the second experiment with "A nudge can only be in one active experiment at a time."
- Guardrail breaches during an experiment but the breach is caused by an external factor (not the variant) → the admin can manually override the auto-pause after review; the override is logged.

---

# UI Design Implementation User Stories (US-41 – US-55)

> Derived from `docs/DESIGN.md` and the mockups in `designs/`. Each story
> covers the visual/UX implementation of a page or subsystem. Functional
> behavior is cross-referenced to existing user stories; these stories focus on
> the UI structure, design-system compliance, and Angular component mapping.

# User Story 41 – Design System Foundation

As a frontend developer, I want a shared design system stylesheet (tokens, typography, and reusable component classes) migrated into the Angular application, so that every page renders with a consistent visual language and new pages can be built from pre-defined primitives.

## Acceptance Criteria

- **Token migration** – All CSS custom properties from `designs/styles.css` are migrated to `frontend/src/styles.scss` as SCSS variables and/or CSS custom properties, including:
  - Background tokens: `--bg`, `--bg-elev`, `--panel`, `--sidebar`
  - Border tokens: `--border`, `--border-strong`
  - Text tokens: `--text`, `--text-muted`, `--text-dim`
  - Status tokens: `--accent`, `--ok`, `--warn`, `--bad`, `--info` (each with a `*-soft` translucent variant)
- **Typography** – Font stacks are defined globally:
  - UI text: `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif`
  - Monospace: `"SF Mono", "JetBrains Mono", Menlo, Consolas, monospace` (used for container IDs, URLs, YAML, env keys, log output)
  - Base size: 14px body, 1.5 line-height; headings step down from 16px (topbar) to 11px uppercase section labels.
- **Reusable components** – The following component classes are implemented as shared Angular components or SCSS mixins under `frontend/src/app/shared/`:
  - Card (`.card`, `.card__head`, `.card__body`, `.card__foot`)
  - Stat tile (`.stat`) — label + large value + delta indicator
  - Button (`.btn`, `.btn--primary`, `.btn--danger`, `.btn--ghost`, `.btn--sm`, `.btn--icon`)
  - Badge (`.badge`, `.badge--ok/warn/bad/info/neutral/accent`) — status pill with colored dot
  - Table (`table.tbl`) — sortable-style header, hover row, `.unhealthy` row turns red
  - Form field (`.field`, `.input`, `.select`, `textarea`)
  - Key/value row (`.kv-row`) — grid for env var / port mapping entry
  - App card (`.app-card`) — store tile with icon, name, category, description, footer
  - Modal (`.modal-overlay`, `.modal`, `.modal__head/body/foot`) — centered dialog with backdrop blur
  - Progress steps (`.progress`, `.progress__step`) — horizontal step indicator
  - Progress bar (`.bar`, `.bar__fill`) — linear meter for CPU/mem/disk
  - Log stream (`.log`) — terminal-style block with colored severity lines
  - Tabs (`.tabs`, `.tab`) — underline tab navigation
  - Avatar (`.avatar`) — gradient circle with initials
- **Design principles** – All pages adhere to the five design principles: operational clarity, density without noise, dark DevOps-native aesthetic, consistent shell, progressive disclosure.

## Technical Constraints & Edge Cases

**Technical Constraints**

- The design system uses a dark theme; all color tokens assume a dark background (`--bg: #0b1020`).
- Each status color must have a matching `*-soft` translucent variant for badge backgrounds (e.g., `--ok-soft = rgba(34,197,94,0.14)`).
- Shared components must be standalone Angular components (matching the existing `frontend/` standalone-component convention) under `frontend/src/app/shared/`.
- The mockups use emoji glyphs as icon placeholders; a real icon set (Material Icons, Lucide, or Heroicons) must be chosen and replaced consistently during implementation.
- `styles.css` tokens must be migrated to SCSS variables or CSS custom properties, not copied as a raw CSS file.

**Edge Cases**

- A page needs a color or spacing value not defined in the token set → extend the token set in `styles.scss` rather than hardcoding; review with the design system owner.
- Monospace font is not installed on the user's system → the font stack falls back to `Menlo, Consolas, monospace` gracefully.
- A reusable component needs a variant not in the mockup (e.g., a warning badge with an icon) → extend the component with a new modifier class following the existing naming convention.
- WCAG 2.1 AA contrast for `--text-dim` on `--bg` must be verified at implementation; adjust if it fails.

# User Story 42 – Application Shell & Navigation Model

As an end-user or administrator, I want a consistent sidebar + topbar application shell with grouped navigation that mirrors the RBAC structure, so that navigation is predictable across all 12+ pages and admin-only routes are visually separated from general routes.

## Acceptance Criteria

- **Layout shell** – All authenticated pages use a two-column grid:
  - **Sidebar** (240px, sticky, full-height): brand logo, grouped navigation, and a user card with avatar, name, and role badge.
  - **Topbar** (56px): breadcrumb, page title, spacer, search input, notifications bell.
  - **Content** (24px padding, max-width 1280px): pages may opt into `content--wide` to drop the max-width for dense tables and grids.
- **Navigation groups** – The sidebar groups routes into two sections mirroring `roles/roles.yaml`:
  - **General** (user + admin): Dashboard, App Store, My Apps, Containers, Launch (admin-only action).
  - **Administration** (admin-only): Users, Apps, Secrets, Backups & Logs, Settings.
- **Active state** – The active nav item is highlighted with `--accent` color and a soft accent background.
- **Login bypass** – The login page (`login.html`) is the only page that bypasses the shell; it uses a centered card on a gradient backdrop.
- **Angular migration** – The existing Angular app shell (`frontend/src/app/app.html`) currently uses a top-nav bar; it must be replaced with the sidebar layout from the mockups.
- **Route additions** – The following routes are added to `app.routes.ts` (currently only `''`, `login`, `apps`, `admin` exist): `my-apps`, `containers`, `launch`, and admin children (`admin/users`, `admin/apps`, `admin/secrets`, `admin/backups`, `admin/settings`).
- **RBAC enforcement** – Admin-only nav items are hidden for non-admin users; navigating to an admin route without permission redirects to the dashboard with an error toast.

## Technical Constraints & Edge Cases

**Technical Constraints**

- The sidebar must be sticky and full-height (`position: sticky; top: 0; height: 100vh`).
- The topbar is 56px tall with breadcrumb, title, spacer, search, and notifications.
- The content area defaults to `max-width: 1280px`; the `content--wide` class removes this for dense tables (containers, admin tables).
- The Angular `App` component (`app.html` / `app.scss`) must be refactored from top-nav to sidebar.
- Admin routes must be lazy-loaded and guarded by an `AdminGuard` that checks the JWT role claim.
- The sidebar nav items must be generated from a route configuration object so that RBAC visibility is data-driven.

**Edge Cases**

- Non-admin user navigates directly to `/admin/secrets` → `AdminGuard` redirects to `/` (dashboard) with an "Access denied" toast.
- Window is resized to a narrow width → the sidebar collapses to icons-only or a hamburger menu (responsive behavior not in mockups; implement per Material/Angular conventions).
- User has no notifications → the bell icon shows no badge; clicking it opens an empty dropdown with "No new notifications."
- Search is triggered from the topbar → it searches across apps, containers, and audit log entries; results are shown in a dropdown or dedicated search page.
- The current Angular app uses a top-nav → the migration to sidebar must not break existing routes (`''`, `login`, `apps`, `admin`).

# User Story 43 – Login Page UI

As an end-user, I want a clean, centered login card on a gradient backdrop with brand identity and a server-status indicator, so that I can authenticate confidently and know the platform is reachable.

## Acceptance Criteria

- **Layout** – Centered card on a blue/purple radial-gradient backdrop; bypasses the application shell.
- **Brand block** – Logo + tagline at the top of the card.
- **Form fields** – Username/email input, password input, "Remember me" checkbox, forgot-password link.
- **Server status** – A "Server online" status badge is rendered on the card.
- **Footer note** – "Protected by JWT · HttpOnly cookie · RBAC enforced" text below the form.
- **Submit** – Clicking "Sign in" sends credentials to `POST /api/v1/auth/login`; on success, a JWT is stored in an HttpOnly cookie and the user is redirected based on role (admin → dashboard, user → app store).
- **Error** – Invalid credentials show a red error banner on the card without clearing the username field.

## Technical Constraints & Edge Cases

**Technical Constraints**

- The login page is the only page that bypasses the sidebar + topbar shell.
- The form must submit to the existing authentication endpoint (US-16 / FR-AUTH-1).
- The JWT must be stored in an HttpOnly cookie (not localStorage).
- The "Server online" badge polls `GET /healthz` on page load; if unreachable, it shows "Server offline" in `--bad`.
- The card must be vertically and horizontally centered on all viewport sizes.

**Edge Cases**

- Server is unreachable → "Server online" badge shows "Server offline" in `--bad`; the submit button is disabled with a tooltip "Cannot reach server."
- User enters correct credentials but the JWT cookie is blocked by browser settings → redirect fails; show "Authentication succeeded but session could not be established — check cookie settings."
- User clicks "Forgot password" → navigates to a password-reset flow (if implemented) or shows a "Contact your administrator" message.
- "Remember me" is unchecked → the JWT cookie has a session-only lifetime; checking it extends the expiry.

# User Story 44 – Dashboard UI

As an end-user or administrator, I want a dashboard landing page with stat tiles, a My Apps preview, system health bars, and a recent activity feed, so that I can assess platform state at a glance after login.

## Acceptance Criteria

- **Stat tiles** – Four `.stat` tiles across the top: Installed Apps, Healthy Containers, Unhealthy, Last Backup. Each tile shows a large value and a delta indicator.
- **My Apps preview** – A table preview (app, status, URL, open) with a "View all" link to `my-apps.html`. Replaced by the N-HYG-1 empty state when zero apps are installed.
- **System Health panel** – CPU, Memory, Disk, Network progress bars (`.bar`, `.bar__fill`) showing current host resource usage.
- **Recent Activity** – An audit feed (time, user, action, resource, outcome) showing the latest events. High-risk events are highlighted with a `--warn`/`--bad` left border per N-HYG-2.
- **Nudge integration** – The dashboard renders `<sam-nudge>` banners (N-SEC-3 backup gap, N-REL-1 unhealthy triage) in the banner slot above the stat tiles.
- **Maps to** – US-3 (health overview), US-21 (last backup), audit logging, US-31 (dashboard entry point).

## Technical Constraints & Edge Cases

**Technical Constraints**

- Stat tiles must support dynamic color: the "Unhealthy" tile turns `--bad` when count > 0; the "Last Backup" tile turns `--warn`/`--bad` per N-SEC-3.
- The "Unhealthy" stat tile is a deep link to `containers.html?health=unhealthy` (N-REL-1).
- The "Last Backup" stat tile includes an inline "Back up now" button when N-SEC-3 fires.
- System Health bars poll host resource usage via `GET /api/v1/system/health` (or equivalent) on load and refresh periodically.
- The recent-activity feed is sourced from the audit log; high-risk event classification follows N-HYG-2 rules.

**Edge Cases**

- Zero apps installed → "My Apps" preview is replaced by the N-HYG-1 guided empty state with three starter apps and a "Set up nightly backups" CTA.
- All containers healthy → "Unhealthy" tile shows 0 in `--ok`; no N-REL-1 nudge fires.
- No backup has ever been taken → "Last Backup" tile shows "Never" in `--bad`; N-SEC-3 fires immediately.
- System Health data is unavailable (Docker SDK error) → bars show "N/A" in `--text-dim`; no crash.
- Recent activity feed is empty (fresh install) → show a "No recent activity" placeholder with a link to the app store.

# User Story 45 – App Store Catalog UI

As an end-user, I want to browse a YAML-driven app catalog with category filters, sort options, and install counts on each card, so that I can discover and install apps in one click.

## Acceptance Criteria

- **Category filter chips** – All, Media, Productivity, Development, Security, Networking, Home. Selecting a chip filters the grid.
- **Sort dropdown** – Sort by popularity (install count), name, recently added.
- **App card grid** – Responsive auto-fill grid of `.app-card` tiles, each with: icon, name, category, description, rating, install count, and an **Install** button.
- **Social-proof line** – Each card footer shows a contextual social-proof line per N-ADOPT-2 (e.g., "Popular in Media — installed by 8 of 10 tenants" or "Recently added" in single-tenant MVP).
- **Install action** – Clicking **Install** opens the install modal (US-46).
- **Maps to** – US-6 / FR-APP-1 (one-click install), US-15 (secure app browsing), N-ADOPT-2 (social proof).

## Technical Constraints & Edge Cases

**Technical Constraints**

- The app card grid must be responsive (CSS `auto-fill` with `minmax` for card width).
- Category chips and sort dropdown filter/sort the grid client-side or via query parameters.
- Install counts are sourced from the backend; in single-tenant MVP, N-ADOPT-2 falls back to global install counts + "Recently added" badge.
- The `.app-card` component must be a standalone Angular presentational component (`AppCardComponent`) with `@Input()` for app data and `@Output()` for the install event.
- Emoji icons are placeholders; replace with a real icon set during implementation.

**Edge Cases**

- No apps match the selected category → show an empty state "No apps in this category yet."
- App catalog fails to load → show an error banner with a "Retry" button.
- An app is disabled by an admin → it does not appear in the store (only enabled apps are listed).
- Install count is 0 → the social-proof line is suppressed (no "installed by 0 of 10 tenants").
- User clicks Install on an app already installed → the button changes to "Installed" with a link to "Open" or "Manage" in My Apps.

# User Story 46 – Install Modal UI

As an end-user, I want a multi-step install modal with parameter configuration, a review step, live install progress, and a post-install summary, so that I can install an app with full visibility into each phase.

## Acceptance Criteria

- **Modal layout** – Modal over a dimmed App Store backdrop with a four-step progress indicator: Configure → Review → Install → Health check.
- **Configure step** – Required parameters: sub-domain (with `.local` suffix preview), admin username, admin password (marked as Docker secret), database select, storage quota. Optional environment-variable key/value rows (add/remove).
- **Review step** – Summary of all configured values; pre-install backup confirmation card.
- **Install step** – Live progress via SSE (FR-RT-1); shows backup → catalog resolve → compose up → health check → audit steps.
- **Health check step** – Waits up to 2 minutes for the container health check to become healthy; shows a spinner and status.
- **Post-install summary** – A summary card with configured values (US-8) and the N-ADOPT-1 backup commitment checkbox ("Remind me to configure backups for {{app}} in 24h").
- **Footer** – Cancel + "Install {{app}} →" button.
- **Maps to** – US-8 / FR-APP-3 (install-time parameter prompting), US-22 (rollback prerequisite — backup), US-25 / FR-RT-1 (live install status), N-ADOPT-1 (backup commitment).

## Technical Constraints & Edge Cases

**Technical Constraints**

- The modal must use the `.modal-overlay` / `.modal` component from the design system with backdrop blur.
- The progress indicator uses `.progress` / `.progress__step` (done/active/pending states).
- Each parameter field must validate (type, regex, length) before allowing advancement to the Review step.
- The Install step subscribes to the SSE channel for live status; the Health check step polls the container health endpoint.
- The post-install summary card must render the N-ADOPT-1 checkbox; if checked, a timed nudge is scheduled.
- The modal must trap focus and be dismissible via Escape or backdrop click (before install starts; not during).

**Edge Cases**

- User clicks Cancel during the Install step → a confirmation warns "Installation in progress — are you sure?" and triggers rollback (US-22) if confirmed.
- Health check times out after 2 minutes → modal shows "Installation Failed – Rolled back" and the backend removes the container (US-6 rollback flow).
- A required parameter is left empty → the "Next" button is disabled; the field shows a validation error.
- Sub-domain is already in use → the Configure step shows an inline error "Sub-domain already taken."
- SSE connection drops during install → the modal falls back to polling `GET /api/v1/installs/{id}/status` and continues showing progress.

# User Story 47 – My Apps Lifecycle UI

As an end-user, I want a My Apps page showing my installed apps with status badges, resource usage, and lifecycle actions (open, pause, restart, uninstall), so that I can manage my app fleet from one place.

## Acceptance Criteria

- **Status filter chips** – All / Running / Stopped / Unhealthy.
- **App cards** – Each card shows: status badge, sub-domain, version, CPU/memory usage, and action buttons (Open, Pause, Restart, Uninstall).
- **Starting state** – A card in "Starting" state shows a "Live status via SSE" note and a Cancel button.
- **Unhealthy state** – A card in "Unhealthy" state has a red border and a "View logs" button.
- **Stopped state** – A card in "Stopped" state shows a primary "Start" action.
- **Uninstall modal** – Clicking Uninstall opens a confirmation modal. For apps with running dependents, the modal lists dependents with health state and requires typed confirmation of the app slug (N-REL-2).
- **Maps to** – US-9 / FR-UNI-1 (easy uninstall), US-25 (live status), US-3 (per-app health visibility), N-REL-2 (dependency-aware confirmation).

## Technical Constraints & Edge Cases

**Technical Constraints**

- App cards must reflect real-time status via SSE (starting, running, stopped, unhealthy).
- The "Open" button links to the app's sub-domain URL in a new tab.
- The Uninstall modal must query running dependents before rendering; if dependents exist, N-REL-2 typed confirmation is required.
- CPU/memory usage on each card is sourced from the container stats endpoint and refreshes periodically.
- Status badges use the design-system `.badge` component with appropriate color modifiers (`--ok`, `--warn`, `--bad`, `--info`).

**Edge Cases**

- App is uninstalled from another session → the card disappears on next SSE update; a toast confirms "App uninstalled."
- User clicks "Pause" on a container that is already stopped → the action is a no-op; the UI shows "Already stopped."
- Uninstall modal opens but dependents are healthy → N-REL-2 still fires (dependent health state is shown regardless).
- Sub-domain URL is unreachable (app crashed) → "Open" button links anyway; the unhealthy border and "View logs" guide the user to diagnose.
- Multiple apps are in "Starting" state simultaneously → each card shows independent SSE status; no card is blocked by another.

# User Story 48 – Container Dashboard UI

As an end-user or administrator, I want a container operations table with health, resource usage, real-time updates, and filtering, so that I can detect and triage container failures quickly.

## Acceptance Criteria

- **Topbar badges** – "Auto-refresh 10s" and "SSE connected" badges in the page topbar.
- **Stat tiles** – Running / Starting / Unhealthy / Stopped container counts.
- **Table columns** – Container ID, Image, Status, Health, CPU %, Mem %, Last checked, Logs action.
- **Unhealthy row highlight** – Rows with Unhealthy health are highlighted red per US-3 acceptance criteria.
- **Filters** – Search input + status dropdown + "Launch" button (admin-only, links to `launch.html`).
- **Auto-refresh** – The table auto-refreshes every 10 seconds without losing the current sort/filter state.
- **Real-time updates** – While the page is open, any change in health/status is reflected instantly via SSE.
- **URL filtering** – The page supports `?health=unhealthy` query parameter to pre-filter on load (deep-linked from N-REL-1 dashboard tile).
- **Logs action** – Clicking "Logs" opens a log stream panel for that container.
- **Maps to** – US-3 / FR-CL-2 (container health dashboard), US-4 / FR-CL-3 (admin container view), US-25 (real-time updates).

## Technical Constraints & Edge Cases

**Technical Constraints**

- The table must use `table.tbl` from the design system with `.unhealthy` row highlighting.
- Auto-refresh (every 10s) must preserve sort order and filter state (no visual jump).
- SSE updates must be deduplicated by container ID; only changed rows re-render.
- The "Launch" button is visible only to admin users (`write:containers` permission).
- URL query parameter `?health=unhealthy` must pre-set the status dropdown filter on page load.
- The "Logs" action opens a side panel or modal with the `.log` terminal-style block streaming container logs.

**Edge Cases**

- SSE disconnects → the "SSE connected" badge flips to "Reconnecting…" in `--warn`; the table falls back to 10s polling.
- No containers exist on the host → the table shows an empty state "No containers running" with a link to the App Store (for end-users) or Launch (for admins).
- A container is removed between refreshes → its row animates out (or simply disappears on next render); no error.
- Sort is applied on a column while SSE updates arrive → sort order is maintained; new rows are inserted in the correct position.
- `?health=unhealthy` is set but no unhealthy containers exist → the table shows an empty filtered state "No unhealthy containers" with a "Clear filter" link.

# User Story 49 – Launch Container Form UI

As an administrator, I want a launch container form with a two-column layout (form + validation/preview side panels), so that I can start arbitrary child containers with full Docker run options and pre-flight validation.

## Acceptance Criteria

- **Two-column layout** – Form on the left; validation and preview side panels on the right.
- **Form fields** – Image, container name, restart policy, env-var key/value rows, port mappings (host:container + protocol), command/entrypoint override.
- **Resource-limit suggestion** – When `mem_limit`/`cpu_quota` are empty, a non-blocking `--info` hint shows the host's current free memory and a suggested limit band; the fields are pre-filled but editable (N-REL-3).
- **Pre-flight checks panel** – Side panel showing real-time validation badges: image pullability, port availability, resource sufficiency, command syntax, network — each with an OK/FAIL badge.
- **Compose preview panel** – Side panel showing the rendered Docker Compose YAML in a `.log` monospace block.
- **Launch action** – Clicking "Launch" sends `POST /containers/launch` with the form payload (US-2).
- **Maps to** – US-2 / FR-CL-1 (launch child containers), US-20 (secret references), US-18 (validation), N-REL-3 (resource-limit suggestion).

## Technical Constraints & Edge Cases

**Technical Constraints**

- The form must use `.field`, `.input`, `.select`, `.kv-row` design-system components.
- Pre-flight checks run client-side where possible (port format regex) and server-side where Docker SDK is needed (image pullability, resource sufficiency).
- The Compose preview updates in real-time as the user edits the form (debounced).
- N-REL-3 hint queries host free memory via `GET /api/v1/system/health` or Docker SDK; the suggestion is a conservative band, not the maximum.
- The "Launch" button is disabled until all pre-flight checks pass (or warnings are acknowledged).

**Edge Cases**

- Image name is invalid or not pullable → pre-flight "image pullability" badge shows FAIL in `--bad`; the Launch button is disabled.
- Port is already in use → pre-flight "port availability" badge shows FAIL; the user must change the port or acknowledge.
- Host has insufficient resources → pre-flight "resource sufficiency" badge shows FAIL; the Launch button is disabled with a message.
- Command contains shell metacharacters → pre-flight "command syntax" badge shows FAIL; validation rejects before the Docker call.
- User overrides the N-REL-3 suggested limit to empty → the hint re-appears; the user can launch without limits but the pre-flight shows a `--warn` "No resource limits set."
- Compose preview YAML has a syntax error → the preview panel shows the error inline; the Launch button is disabled.

# User Story 50 – Admin – Users & Roles UI

As an administrator, I want a user management page with tabs for Users, Roles & permissions, and Audit log, so that I can manage accounts, inspect the permission matrix, and review security-relevant actions.

## Acceptance Criteria

- **Tabs** – Users / Roles & permissions / Audit log.
- **Users table** – Avatar + name + email, role badge (admin/user), status (Active/Deactivated), last login, created date, Edit action.
- **Add/Edit user dialog** – Role `<select>` defaults to `user` (least privilege) with helper text showing the permission delta vs. `admin` in plain language (N-SEC-2).
- **Roles & permissions table** – Reference table mapping permissions (`apps.install`, `apps.uninstall.own/any`, `containers.launch`, `users.manage`, `secrets.manage`, `settings.manage`) to admin vs. user, sourced from `roles/roles.yaml`.
- **Audit log tab** – Filterable, paginated audit log of all admin actions and nudge events.
- **Maps to** – US-13 / FR-AUTH-3 (admin user management), US-11/15 / FR-AUTH-2 (role-based permissions), audit logging, N-SEC-2 (strong defaults).

## Technical Constraints & Edge Cases

**Technical Constraints**

- The Users table uses `table.tbl` with avatar (`.avatar`), role badge (`.badge`), and status badge.
- The Add/Edit user dialog must default the role `<select>` to `user` (N-SEC-2); the permission delta helper text is derived from `roles/roles.yaml`.
- The Roles & permissions table is read-only ( sourced from `roles/roles.yaml`); editing the YAML is done via `admin-apps.html` or a dedicated roles editor (future).
- The Audit log tab supports filtering by user, action type, resource, and date range; pagination is server-side.
- Audit log entries from the BNE are tagged `source: 'bne'` and can be filtered.

**Edge Cases**

- Admin tries to deactivate their own account → the action is blocked with "You cannot deactivate your own account."
- Admin tries to change their own role from admin to user → the action is blocked with "You cannot remove your own admin privileges."
- `roles/roles.yaml` is missing or invalid → the Roles & permissions table shows an error "Could not load roles configuration."
- Audit log has > 10,000 entries → server-side pagination with a "Load more" button; no client-side rendering of all rows.
- A user has never logged in → "Last login" shows "Never" in `--text-dim`.

# User Story 51 – Admin – Apps & YAML Editor UI

As an administrator, I want an app definition management page with tabs for Catalog, Enabled, and Disabled apps, plus an inline YAML editor with schema validation, so that I can enable, disable, delete, and edit app definitions.

## Acceptance Criteria

- **Tabs** – Catalog (count) / Enabled (count) / Disabled (count).
- **Apps table** – App icon + name, version, category, state badge, install count, YAML path, Edit/Disable (or Enable) actions.
- **Stale version badge** – When an enabled app's YAML version is older than the store version, a row-level `--info` "Update available" badge appears with a one-click "Diff & upgrade" action (N-ADOPT-3).
- **YAML editor card** – Shows the app's YAML (e.g., `store/nextcloud.yaml`) with syntax-highlighted placeholders (`{{ subdomain }}`), param annotations, and an "Unsaved changes" badge.
- **Editor actions** – Discard, Validate schema, Save.
- **Schema validation** – Clicking "Validate schema" runs the YAML against the JSON schema (US-7); errors are listed inline.
- **Diff & upgrade** – Clicking "Diff & upgrade" (N-ADOPT-3) opens the editor pre-loaded with the new version and a highlighted diff.
- **Maps to** – US-12 / FR-ADMIN-1 (admin app management), US-7/14 / FR-APP-2 (YAML definitions), US-18 (schema validation), N-ADOPT-3 (stale definition nudge).

## Technical Constraints & Edge Cases

**Technical Constraints**

- The YAML editor must render in a monospace `.log`-style block with syntax highlighting for YAML and placeholder tokens.
- "Unsaved changes" badge (`.badge--warn`) appears when the editor content diverges from the saved file.
- "Validate schema" calls the backend schema validation endpoint (US-18); errors are mapped to line numbers in the editor.
- "Diff & upgrade" loads the store version of the YAML and renders a highlighted diff (additions in `--ok`, removals in `--bad`).
- The table uses `table.tbl` with state badges (`.badge--ok` for enabled, `.badge--neutral` for disabled).
- Tab counts are dynamic, sourced from the backend.

**Edge Cases**

- YAML has a syntax error → "Validate schema" returns parsing errors with line numbers; "Save" is disabled.
- Admin edits the YAML and navigates away without saving → a "You have unsaved changes" confirmation dialog appears.
- "Diff & upgrade" is clicked but the new version has breaking schema changes → the editor surfaces a validation warning before save.
- An app is currently installed and the admin disables it → a confirmation warns "X instances are running — they will be stopped"; confirming triggers the disable + stop flow.
- YAML file is not found on disk → the editor shows "File not found" in `--bad`; Save is disabled.

# User Story 52 – Admin – Secrets Management UI

As an administrator, I want a Docker secrets CRUD page with a hard rule that secret values are never exposed after creation, so that I can manage sensitive credentials securely.

## Acceptance Criteria

- **Warning banner** – A persistent warning banner at the top: "Secret values are never exposed; rotate to set a new value."
- **Secrets table** – Name (mono), scope badge, used-by count, created date, last rotated date, Rotate + Delete actions.
- **Rotation reminder banner** – When one or more secrets exceed the rotation threshold (default 90 days) or have `last_rotated_at = null`, a `--warn` banner appears above the table listing expiring keys with a "Rotate selected" button (N-SEC-1).
- **New secret modal** – Name, scope select, value textarea with a hint that it is encrypted at rest and unretrievable.
- **Rotate action** – Clicking "Rotate" opens a modal to set a new value; the old value is permanently replaced.
- **Delete action** – Clicking "Delete" requires typed confirmation of the secret name; deletion is logged in the audit log.
- **Maps to** – US-20 / FR-ADMIN-2 (Docker secrets management), NF-SEC, N-SEC-1 (rotation reminder).

## Technical Constraints & Edge Cases

**Technical Constraints**

- Secret values must never be returned by any API endpoint after creation; only metadata (name, scope, created, last_rotated, used-by) is shown.
- The secrets table uses `table.tbl` with monospace name column and `.badge` for scope.
- N-SEC-1 banner is rendered via `<sam-nudge>` in the banner slot above the table; it lists expiring keys and includes a "Rotate selected" button that pre-fills the rotation modal.
- The "New secret" modal uses `.modal` with a textarea for the value; the value is encrypted at rest.
- Delete confirmation requires exact typed match of the secret name (case-sensitive).
- All actions (create, rotate, delete) are logged in the audit log.

**Edge Cases**

- Admin tries to retrieve a secret value → the API returns 403 "Secret values are not retrievable"; the UI never offers a "view value" action.
- Secret is referenced by a running container and admin deletes it → a warning lists the containers using it; deletion requires explicit confirmation "X containers are using this secret."
- All secrets are within the rotation threshold → N-SEC-1 banner is not shown.
- Secret name contains special characters → the typed delete confirmation must match exactly, including special characters.
- `last_rotated_at` is null (secret never rotated) → N-SEC-1 lists it as expiring regardless of created date.

# User Story 53 – Admin – Backups & Logs UI

As an administrator, I want a backups and logs page with tabs for Backups, Audit log, and Live logs, so that I can inspect backup history, review audit events, and tail live platform logs in real time.

## Acceptance Criteria

- **Tabs** – Backups / Audit log / Live logs.
- **Stat tiles** – Total backups, Success rate, Next nightly (time in UTC).
- **Backup history table** – Timestamp, type (Full / Pre-install), trigger, size, status badge, Restore + Download actions. Header notes retention (e.g., 14 days) and disk usage (e.g., 142 GB).
- **Audit log tab** – Filterable, paginated audit log (same data as US-50 audit tab, with additional backup-specific filters).
- **Live log stream** – Terminal-style `.log` block with timestamped, severity-colored lines (INFO/OK/WARN/ERROR) showing real-time platform activity (install flows, backup events, health checks, audit events).
- **Restore action** – Clicking "Restore" on a backup row opens a confirmation modal; restoring triggers the rollback flow (US-22).
- **Download action** – Clicking "Download" downloads the backup archive.
- **Maps to** – US-21 / FR-ADMIN-3 (backup & log settings), US-29 (log streaming), US-22 (pre-install backup evidence), audit logging.

## Technical Constraints & Edge Cases

**Technical Constraints**

- The live log stream uses the `.log` component with `aria-live="polite"` for accessibility.
- Log lines are color-coded by severity: INFO in `--text-muted`, OK in `--ok`, WARN in `--warn`, ERROR in `--bad`.
- The live log stream connects via SSE; on disconnect, it shows "Stream paused — reconnecting…" and resumes automatically.
- The backup history table uses `table.tbl` with `.badge` for status (Success = `--ok`, Failed = `--bad`, In progress = `--warn`).
- Restore confirmation modal warns about data overwrite; typed confirmation of the backup timestamp may be required for full restores.
- Download action streams the backup file from the backend; large files show a progress indicator.

**Edge Cases**

- No backups exist → the table shows an empty state "No backups yet" with a "Back up now" button.
- Live log stream produces output faster than the UI can render → lines are buffered and throttled; a "Showing last N lines" indicator appears.
- Backup restore fails (corrupt archive) → the confirmation modal shows an error; the audit log records the failed restore attempt.
- Disk usage exceeds retention capacity → a `--warn` banner appears "Backup disk almost full — review retention settings" with a link to admin settings.
- SSE disconnects during live log streaming → the stream pauses and resumes on reconnect; no log lines are lost (backend buffers).

# User Story 54 – Admin – Settings UI

As an administrator, I want a global settings page with tabs for General, SSL/TLS, Backups, Rate limiting, and Security, so that I can configure all platform-wide operational parameters in one place.

## Acceptance Criteria

- **Tabs** – General / SSL / TLS / Backups / Rate limiting / Security.
- **General tab** – Platform name, base domain, default language (English / Nederlands per US-31 i18n).
- **SSL / TLS tab** – Certificate mode (Let's Encrypt / custom / self-signed), ACME email, DNS challenge provider, custom cert upload. "ACME active" badge when Let's Encrypt is enabled.
- **Backups tab** – Nightly time (UTC), retention days, backup path, pre-install snapshot toggle.
- **Rate limiting tab** – Installs/user/hour, installs/app/hour, login attempts/5 min, API requests/min.
- **Security tab** – Grid showing CSP, X-Frame-Options, HSTS, Referrer-Policy, CORS (all On) and CSRF (Off, with an "Enable CSRF" button). Security headers cannot be turned off via this UI (US-32).
- **Save behavior** – Each tab has independent Save/Discard; unsaved changes show a `.badge--warn` "Unsaved changes" indicator.
- **Maps to** – US-24 / FR-SSL-1 (SSL management), US-21 / FR-ADMIN-3 (backup schedule), US-25 / FR-RT-2 (rate limiting), US-5/26/32 / NF-SEC (security headers), US-31 / NF-A11Y (i18n).

## Technical Constraints & Edge Cases

**Technical Constraints**

- Each tab is a routed child or a tabbed view within the Admin Settings component.
- Settings are persisted via `PATCH /api/v1/admin/settings` (or per-section endpoints); all changes are audited.
- The SSL/TLS tab must support three modes: Let's Encrypt (ACME), custom cert upload, and self-signed.
- Rate limiting fields must validate numeric ranges (e.g., installs/user/hour > 0).
- The Security tab is read-only for header status (headers are enforced by middleware, US-32); the only actionable control is the CSRF toggle.
- Language selection (General tab) sets the `@ngx-translate` default language (en/nl per US-31).

**Edge Cases**

- Admin switches SSL mode from Let's Encrypt to custom → a warning appears "Existing ACME certificates will not be renewed"; confirming triggers the switch.
- Custom cert upload fails (invalid format) → the upload shows an error "Invalid certificate format"; the mode is not changed.
- Retention days is set to 0 → validation rejects with "Retention must be at least 1 day."
- Admin navigates between tabs with unsaved changes → a "You have unsaved changes" confirmation appears.
- CSRF is toggled from Off to On → a warning explains that all state-changing requests will require CSRF tokens; the Angular HTTP interceptor must be updated accordingly.
- Rate limit value exceeds a safe maximum → validation warns "This value may allow abuse — are you sure?" but does not block.

# User Story 55 – Design Accessibility & i18n Compliance

As an accessibility-conscious developer, I want all design mockup pages implemented with WCAG 2.1 AA compliance, ARIA roles, visible focus styles, and full string translation, so that the platform is usable by everyone and ready for internationalization.

## Acceptance Criteria

- **ARIA roles** – The sidebar is marked `role="navigation"`; tabs use `tablist`/`tab`/`tabpanel`; modals use `role="dialog"` with `aria-modal="true"` and a focus trap.
- **Live regions** – The live log stream and SSE status badges use `aria-live="polite"` so screen readers announce updates.
- **Focus styles** – Visible `:focus` box-shadows are applied to all interactive elements (inputs, buttons, links, nav items), extending the existing input focus styles from the design system.
- **Color contrast** – All text/background pairs meet WCAG 2.1 AA contrast. The `--text-dim` on `--bg` pair must be verified at implementation and adjusted if it fails.
- **Keyboard navigation** – All interactive elements are reachable via Tab; modals trap focus; Escape closes modals; Enter activates buttons and links.
- **Translation** – All visible strings are translated via `@ngx-translate` with `en` and `nl` locales configured (per US-31). No hardcoded user-facing strings in components.
- **Empty/loading/error states** – Implementation includes skeleton loaders, empty-state illustrations, and error banners consistent with the design system (not in scope for the mockups but required for production).

## Technical Constraints & Edge Cases

**Technical Constraints**

- ARIA roles must be applied at the Angular component template level, not via runtime DOM manipulation.
- The focus trap in modals must cycle focus within the modal while open and restore focus to the triggering element on close.
- `@ngx-translate` is already configured in the frontend (per US-31); all new components must use translation keys, not inline strings.
- Skeleton loaders must match the layout of the loaded content (same dimensions) to prevent layout shift.
- Error banners must use the `.badge--bad` / `--bad` color tokens and be announced via `aria-live="assertive"`.
- WCAG 2.1 AA contrast must be verified with an automated tool (e.g., axe-core) in CI.

**Edge Cases**

- Screen reader user navigates the container dashboard table → table headers must use `<th scope="col">` for proper column association.
- Keyboard user tabs through the app card grid → each card is a single tab stop with internal actions accessible via Enter/Space.
- Modal is opened and user presses Tab past the last focusable element → focus wraps to the first focusable element (focus trap).
- Translation key is missing for `nl` locale → the translation service falls back to `en` and logs a warning.
- Skeleton loader is shown for > 3 seconds → a "Taking longer than expected…" message appears below the skeleton.
- Automated contrast check fails for `--text-dim` on `--bg` → the token value is adjusted (lightened) until AA passes; the change is documented in the design system.
