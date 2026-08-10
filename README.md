# Server App Manager

A single-tenant, self-hosted "app store" for containerized applications. Deploy, manage, and monitor Docker apps through an Angular UI and a Rust-backed REST API, with automated sub-domain provisioning, SSL termination, secrets management, backups, and rollbacks.

## Architecture

Three-tier containerized stack:

| Tier         | Technology                          |
| ------------ | ----------------------------------- |
| Presentation | Angular SPA (served by the backend) |
| Application  | Rust REST API (axum)                |
| Infrastructure | Docker Compose, Traefik, Postgres |

The Rust backend serves both the `/api/v1` REST API and the compiled Angular SPA via `tower-http` `ServeDir`. Traefik handles TLS termination and routing. Postgres stores users and roles.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the full architecture document.

## Prerequisites

- **Docker** 24+ and **Docker Compose** v2+
- **Rust** 1.94+ (for local backend development)
- **Node.js** 24.15+ (for local frontend development; managed via [Volta](https://volta.sh))

## Quick Start

```bash
# 1. Copy the environment template
cp .env.example .env

# 2. Build all images (backend multi-stage build compiles the Angular SPA)
docker compose build

# 3. Start the stack
docker compose up -d

# 4. Verify the backend is healthy
curl http://localhost/healthz
# → {"status":"ok"}

# 5. Verify the API stub
curl http://localhost/api/v1/apps
# → []

# 6. Open the UI in your browser
open http://localhost
```

## Commands

| Command                          | Description                          |
| -------------------------------- | ------------------------------------ |
| `cp .env.example .env`          | Create environment configuration     |
| `docker compose build`          | Build all Docker images              |
| `docker compose up -d`          | Start the stack in detached mode     |
| `docker compose down`           | Stop and remove containers           |
| `docker compose down -v`        | Stop and remove containers + volumes |
| `docker compose logs -f backend`| Follow backend logs                  |
| `docker compose ps`             | Show running services                |

## Local Development

### Backend

```bash
cd backend
cargo build          # debug build
cargo test           # run tests
cargo run --release  # run the server (requires DATABASE_URL env)
```

### Frontend

```bash
cd frontend
npm ci               # install dependencies
npm start            # dev server at http://localhost:4200
npm run build        # production build → dist/frontend/browser/
```

## Project Structure

```
server-app-manager/
├── backend/          Rust axum API + SPA static host
├── frontend/         Angular SPA source
├── traefik/          Traefik static + dynamic config
├── apps/             App YAML definitions (store/enabled/disabled)
├── roles/            Role-permission mapping (roles.yaml)
├── schemas/          JSON Schema for app definitions
├── api/              OpenAPI 3.0 spec (swagger.yaml)
├── docs/             Architecture, PRD, and user stories
├── .github/workflows/ CI pipeline
└── docker-compose.yml
```

## Configuration

See [`.env.example`](.env.example) for all environment variables:

| Variable           | Default                          | Description                          |
| ------------------ | -------------------------------- | ------------------------------------ |
| `TENANT_ID`        | `tenant_default`                 | Prefix for all Docker resources      |
| `DATABASE_URL`     | `postgres://sam:sam@db:5432/sam` | Postgres connection string           |
| `JWT_SECRET`       | `changeme...`                    | JWT signing secret (override in prod)|
| `CORS_ORIGIN`      | `https://app.example.com`        | Allowed CORS origin for the SPA      |
| `ACME_PRODUCTION`  | `false`                          | Use Let's Encrypt production server  |
| `ACME_EMAIL`       | `admin@example.com`              | Let's Encrypt notification email     |

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Product Requirements](docs/PRD.md)
- [User Stories](docs/USER_STORIES.md)
- [User Stories Extras](docs/USER_STORIES_EXTRAS.md)
- [OpenAPI Spec](api/swagger.yaml)

## Status

This is a **skeleton bootstrap**. The stack builds and runs with stub API endpoints and a placeholder Angular UI. Feature implementation (auth, app store, install engine, dashboard, admin panels) is planned as follow-up work per the PRD priorities.
