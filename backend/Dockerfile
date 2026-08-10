# syntax=docker/dockerfile:1.7

# ---- Stage 1: build the Angular SPA -----------------------------------------
FROM node:24-alpine AS frontend-build

WORKDIR /app/frontend
RUN corepack enable
COPY frontend/package.json frontend/pnpm-lock.yaml* frontend/pnpm-workspace.yaml* ./
RUN pnpm install --frozen-lockfile

COPY frontend/ ./
RUN pnpm run build
# Output: /app/frontend/dist/frontend/browser/

# ---- Stage 2: build the Rust backend ----------------------------------------
FROM rust:1.94-slim AS backend-build

WORKDIR /app/backend
ENV CARGO_TERM_COLOR=always
ENV SQLX_OFFLINE=true

# Install build dependencies for sqlx/bollard where needed.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY backend/Cargo.toml backend/Cargo.lock* ./
# Pre-fetch deps by creating a stub crate, then build.
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release || true
RUN rm -rf src

COPY backend/ ./
RUN cargo build --release

# ---- Stage 3: runtime -------------------------------------------------------
FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app

COPY --from=backend-build /app/backend/target/release/appforge-backend /app/appforge-backend
COPY --from=backend-build /app/backend/migrations /app/migrations
COPY --from=frontend-build /app/frontend/dist/frontend/browser /app/static

ENV STATIC_DIR=/app/static
ENV RUST_LOG=info
EXPOSE 8080

CMD ["/app/appforge-backend"]
