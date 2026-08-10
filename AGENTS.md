# Server App Manager – Agent Notes

Build, test, and verification commands for future sessions.

## Root orchestration (run from repo root)

- Install root tooling: `pnpm install` (adds `concurrently`)
- Install frontend deps: `pnpm run install:all`
- Start both (dev): `pnpm run dev` — runs backend + frontend concurrently
- Build both: `pnpm run build`
- Test both: `pnpm test`
- Clean: `pnpm run clean`

## Backend (Rust / axum)

- Build: `cargo build --release` (run from `backend/`)
- Test: `cargo test` (run from `backend/`)
- Run locally: `cargo run --release` (requires `DATABASE_URL` env or `.env`)
- Migrations: `cargo install sqlx-cli --no-default-features --features postgres && sqlx migrate run`

## Frontend (Angular)

- Install: `cd frontend && pnpm install --frozen-lockfile`
- Build: `cd frontend && pnpm run build` (outputs `dist/frontend/browser/`)
- Dev server: `cd frontend && pnpm start`
- Node requirement: v24.15.0+ (pinned via Volta in `frontend/`)
- Package manager: pnpm 11.0.9 (pinned via `packageManager` field)

## Docker Compose stack (production-like)

- Build: `docker compose build`
- Start: `docker compose up -d`
- Stop / cleanup: `docker compose down` (add `-v` to remove volumes)
- Logs: `docker compose logs -f backend`
- Health check: `curl http://localhost/healthz`

## Docker Compose dev stack (hot reload)

- Prereq: `cp .env.dev.example .env.dev`
- Start: `pnpm run dev:docker:up` (or `docker compose -f docker-compose.dev.yml --env-file .env.dev up -d --build`)
- Logs: `docker compose -f docker-compose.dev.yml logs -f backend frontend`
- Stop: `pnpm run dev:docker:down`
- Full reset (wipes DB): `pnpm run dev:docker:reset`
- Frontend (HMR): http://localhost:4200 (proxies /api → backend)
- Backend API: http://localhost:8080
- Postgres: localhost:5432 (sam/sam/sam)
- Hot reload: backend uses cargo-watch, frontend uses ng serve --hmr

## CI / Lint

- OpenAPI spec: `swagger-cli validate api/swagger.yaml`
- YAML lint: `yamllint roles/roles.yaml traefik/ docker-compose.yml`
- Security scan: `trivy image --severity HIGH,CRITICAL <image>`

## Architecture reference

See `docs/ARCHITECTURE.md` for the full target architecture, component model,
runtime views, and constraint/edge-case handling matrix.
