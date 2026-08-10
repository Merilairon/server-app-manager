# Server App Manager – Agent Notes

Build, test, and verification commands for future sessions.

## Backend (Rust / axum)

- Build: `cargo build --release` (run from `backend/`)
- Test: `cargo test` (run from `backend/`)
- Run locally: `cargo run --release` (requires `DATABASE_URL` env or `.env`)
- Migrations: `cargo install sqlx-cli --no-default-features --features postgres && sqlx migrate run`

## Frontend (Angular)

- Install: `cd frontend && npm ci`
- Build: `cd frontend && npm run build` (outputs `dist/frontend/browser/`)
- Dev server: `cd frontend && npm start`
- Node requirement: v24.15.0+ (pinned via Volta in `frontend/`)

## Docker Compose stack

- Build: `docker compose build`
- Start: `docker compose up -d`
- Stop / cleanup: `docker compose down` (add `-v` to remove volumes)
- Logs: `docker compose logs -f backend`
- Health check: `curl http://localhost/healthz`

## CI / Lint

- OpenAPI spec: `swagger-cli validate api/swagger.yaml`
- YAML lint: `yamllint roles/roles.yaml traefik/ docker-compose.yml`
- Security scan: `trivy image --severity HIGH,CRITICAL <image>`

## Architecture reference

See `docs/ARCHITECTURE.md` for the full target architecture, component model,
runtime views, and constraint/edge-case handling matrix.
